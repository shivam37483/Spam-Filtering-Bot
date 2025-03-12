/// A module for managing spam detection rules and sender scores using a SQLite database
/// and custom Lua-based rule evaluation.
///
/// This module provides a `RuleManager` struct that handles the storage and application
/// of spam detection rules, as well as tracking sender behavior through a database.
/// It uses `rusqlite` for database operations, `std::sync` for thread-safe access,
/// and `rlua` for executing Lua scripts to evaluate custom rules.
/// 
use rlua::Lua;
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

/// Represents a single spam detection rule consisting of a keyword and an associated score.
///
/// The `Rule` struct is used to define patterns (keywords) and their corresponding
/// spam scores, which are evaluated against messages to determine spam likelihood.
/// It is marked as `Clone` to allow easy duplication of rule instances.
#[derive(Clone)]
pub struct Rule {
    /// The keyword or pattern to match against messages (e.g., "spam", "http").
    pub keyword: String,
    /// The score associated with the keyword, indicating its spam weight (e.g., 10.0 for "spam").
    pub score: f32,
}

/// Manages spam detection rules and sender scores using a SQLite database.
///
/// The `RuleManager` struct maintains a thread-safe connection to a SQLite database
/// and an in-memory cache of rules. It provides methods to initialize the database,
/// add rules, update sender scores, retrieve sender scores, and evaluate messages
/// against custom rules defined in a Lua script.
pub struct RuleManager {
    /// A thread-safe wrapper around the SQLite database connection.
    ///
    /// The `Mutex` ensures that database operations are performed safely in a
    /// multi-threaded environment, while the `Connection` handles SQL queries.
    pub conn: Mutex<Connection>,
    /// A thread-safe cache of active rules loaded from the database.
    ///
    /// The `Arc<Mutex<Vec<Rule>>>` allows shared ownership and safe mutation of
    /// the rule list across threads.
    pub rules: Arc<Mutex<Vec<Rule>>>,
}

impl RuleManager {
    /// Creates a new `RuleManager` instance with the specified database path.
    ///
    /// Initializes a SQLite database connection and creates the necessary tables
    /// (`rules` and `senders`) if they do not exist. Loads existing rules from
    /// the database into an in-memory cache.
    ///
    /// # Arguments
    /// * `db_path` - The file path to the SQLite database (e.g., "rules.db").
    ///
    /// # Returns
    /// * `Result<Self>` - A `Result` containing the new `RuleManager` instance
    ///   on success, or a `rusqlite::Error` if database operations fail.
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

    /// Adds a new rule to the database and in-memory cache.
    ///
    /// Inserts the specified keyword and score into the `rules` table and
    /// updates the in-memory rule cache. If the keyword already exists,
    /// it will be duplicated in the cache (no uniqueness constraint).
    ///
    /// # Arguments
    /// * `keyword` - The keyword to match against messages.
    /// * `score` - The spam score associated with the keyword.
    ///
    /// # Returns
    /// * `Result<()>` - A `Result` indicating success or a `rusqlite::Error`
    ///   if the database operation fails.
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

    /// Increments the spam score for a sender based on message type.
    ///
    /// Updates the `senders` table by incrementing the `spam_score` by 1
    /// if the message is spam, or by 0 (no change) if it is not spam.
    /// Also increments the `message_count` for the sender.
    ///
    /// # Arguments
    /// * `user_id` - The unique identifier of the sender.
    /// * `is_spam` - A boolean indicating whether the message is spam.
    ///
    /// # Returns
    /// * `Result<()>` - A `Result` indicating success or a `rusqlite::Error`
    ///   if the database operation fails.
    pub fn increment_sender_score(&self, user_id: &str, is_spam: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let increment = if is_spam { 1 } else { 0 }; // Only increment for spam, don’t decrement
        conn.execute(
            "INSERT INTO senders (user_id, spam_score, message_count)
                 VALUES (?1, ?2, 1)
                 ON CONFLICT(user_id) DO UPDATE
                 SET spam_score = spam_score + ?2, message_count = message_count + 1",
            &[&user_id, &increment.to_string()[..]],
        )?;
        Ok(())
    }

    /// Retrieves the current spam score for a given sender.
    ///
    /// Queries the `senders` table to get the `spam_score` for the specified
    /// `user_id`. Returns 0 if no record exists for the user.
    ///
    /// # Arguments
    /// * `user_id` - The unique identifier of the sender.
    ///
    /// # Returns
    /// * `i32` - The sender's current spam score, or 0 if not found.
    pub fn get_sender_score(&self, user_id: &str) -> i32 {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT spam_score FROM senders WHERE user_id = ?1")
            .unwrap();
        stmt.query_row(&[user_id], |row| row.get(0)).unwrap_or(0)
    }

    /// Evaluates a message against custom rules defined in a Lua script.
    ///
    /// Loads the `rules.lua` script and executes the `check_spam` function
    /// with the provided message. Returns the total score based on matching
    /// keywords. Logs an error and returns 0.0 if the script fails to load.
    ///
    /// # Arguments
    /// * `message` - The text message to evaluate for spam.
    ///
    /// # Returns
    /// * `f32` - The cumulative spam score for the message, or 0.0 on error.
    pub fn check_custom_rules(&self, message: &str) -> f32 {
        let lua = Lua::new();
        let score: f32 = lua
            .context(|lua_ctx| {
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
            })
            .unwrap_or(0.0);
        score
    }
}

/// Unit tests for the `rules` module.
///
/// These tests cover the core functionality of `RuleManager`, including
/// database initialization, rule addition, sender score updates, score retrieval,
/// and custom rule evaluation.
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    use RuleManager;

    /// Sets up a temporary database and RuleManager for testing.
    fn setup_test_manager() -> (NamedTempFile, RuleManager) {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = RuleManager::new(temp_file.path().to_str().unwrap()).unwrap();
        (temp_file, manager)
    }

    #[test]
    fn test_new_initializes_database() {
        let (temp_file, manager) = setup_test_manager();
        // Verify tables are created by attempting to insert and query
        let conn = manager.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO rules (keyword, score) VALUES (?1, ?2)",
            &["test", &"5.0"],
        )
        .unwrap();
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM rules", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_add_rule_succeeds() {
        let (temp_file, manager) = setup_test_manager();
        let result = manager.add_rule("spam".to_string(), 10.0);
        assert!(result.is_ok());
        let rules = manager.rules.lock().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].keyword, "spam");
        assert_eq!(rules[0].score, 10.0);
    }

    #[test]
    fn test_increment_sender_score() {
        let (temp_file, manager) = setup_test_manager();
        let result = manager.increment_sender_score("user1", true);
        assert!(result.is_ok());
        let score = manager.get_sender_score("user1");
        assert_eq!(score, 1);
        let result = manager.increment_sender_score("user1", false);
        assert!(result.is_ok());
        let score = manager.get_sender_score("user1");
        assert_eq!(score, 1); // No decrement
    }

    #[test]
    fn test_get_sender_score_returns_zero_for_new_user() {
        let (temp_file, manager) = setup_test_manager();
        let score = manager.get_sender_score("nonexistent");
        assert_eq!(score, 0);
    }

    #[test]
    fn test_check_custom_rules() {
        let (temp_file, manager) = setup_test_manager();
        // Create a temporary rules.lua for testing
        let lua_content = r#"
            function check_spam(message)
                if string.lower(message):find("spam") then
                    return 10
                end
                return 0
            end
        "#;
        fs::write("rules.lua", lua_content).unwrap();
        let score = manager.check_custom_rules("This is spam");
        assert_eq!(score, 10.0);
        let score = manager.check_custom_rules("hello");
        assert_eq!(score, 0.0);
    }
}
