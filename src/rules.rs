use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};
use rlua::Lua;

#[derive(Clone)]
pub struct Rule {
    pub keyword: String,
    pub score: f32,
}

pub struct RuleManager {
    conn: Mutex<Connection>,
    rules: Arc<Mutex<Vec<Rule>>>,
}

impl RuleManager {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Mutex::new(Connection::open(db_path)?);
        {
            let conn = conn.lock().unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS rules (
                    id INTEGER PRIMARY KEY,
                    keyword TEXT NOT NULL,
                    score REAL NOT NULL
                )",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS senders (
                    user_id TEXT PRIMARY KEY,
                    spam_score INTEGER DEFAULT 0,
                    message_count INTEGER DEFAULT 0
                )",
                [],
            )?;
        }
        let rules = {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT keyword, score FROM rules")?;
            let rule_iter = stmt.query_map([], |row| {
                Ok(Rule {
                    keyword: row.get(0)?,
                    score: row.get(1)?,
                })
            })?;
            rule_iter.collect::<Result<Vec<_>>>()?
        };
        Ok(Self {
            conn,
            rules: Arc::new(Mutex::new(rules)),
        })
    }

    pub fn add_rule(&self, keyword: String, score: f32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO rules (keyword, score) VALUES (?1, ?2)",
            &[&keyword, &score.to_string()],
        )?;
        let mut rules = self.rules.lock().unwrap();
        rules.push(Rule { keyword, score });
        Ok(())
    }

    pub fn increment_sender_score(&self, user_id: &str, is_spam: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let increment = if is_spam { 1 } else { -1 };
        conn.execute(
            "INSERT INTO senders (user_id, spam_score, message_count)
             VALUES (?1, ?2, 1)
             ON CONFLICT(user_id) DO UPDATE
             SET spam_score = spam_score + ?2, message_count = message_count + 1",
            &[&user_id, &increment.to_string()[..]],
        )?;
        Ok(())
    }

    pub fn get_sender_score(&self, user_id: &str) -> i32 {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT spam_score FROM senders WHERE user_id = ?1").unwrap();
        stmt.query_row(&[user_id], |row| row.get(0)).unwrap_or(0)
    }

    pub fn check_custom_rules(&self, message: &str) -> f32 {
        let lua = Lua::new();
        let score: f32 = lua.context(|lua_ctx| {
            let script = match std::fs::read_to_string("rules.lua") {
                Ok(content) => content,
                Err(e) => {
                    log::error!("Failed to read rules.lua: {}", e);
                    return Ok::<f32, rlua::Error>(0.0);
                }
            };
            lua_ctx.load(&script).exec()?;
            let globals = lua_ctx.globals();
            globals.set("message", message)?;
            let result: f32 = lua_ctx.load("return check_spam(message)").eval()?;
            Ok(result)
        }).unwrap_or(0.0);
        score
    }
}