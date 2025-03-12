use teloxide::{prelude::*, utils::command::BotCommands};
use dotenv::dotenv;
use std::sync::Arc;
use spam_bot_mvp::rules::RuleManager;
use spam_bot_mvp::utils::{is_admin, notify_admins};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Bot commands")]
enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Report a message as spam")]
    Report,
    #[command(description = "Add a custom spam rule (admin only, format: /add_rule <keyword> <score>)")]
    AddRule(String),
}

async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    rule_manager: Arc<RuleManager>,
) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::Start => {
            bot.send_message(msg.chat.id, "Hello! I'm a spam filter bot.").await?;
        }
        Command::Report => {
            if let Some(reply) = msg.reply_to_message() {
                let text = reply.text().unwrap_or("(non-text message)");
                let is_spam = rule_manager.check_custom_rules(text) >= 5.0;
                bot.send_message(msg.chat.id, format!("Reported: {}\nSpam: {}", text, is_spam)).await?;
                if is_spam {
                    let user_id = reply.from().unwrap().id.to_string();
                    if let Err(e) = rule_manager.increment_sender_score(&user_id, true) {
                        log::error!("Failed to update sender score: {}", e);
                    }
                    notify_admins(&bot, msg.chat.id, text, &rule_manager, &user_id).await?;
                }
            } else {
                bot.send_message(msg.chat.id, "Please reply to a message to report it.").await?;
            }
        }
        Command::AddRule(args) => {
            if is_admin(&bot, &msg).await.unwrap_or(false) {
                let parts: Vec<&str> = args.split_whitespace().collect();
                if parts.len() == 2 {
                    let keyword = parts[0].to_string();
                    if let Ok(score) = parts[1].parse::<f32>() {
                        if let Err(e) = rule_manager.add_rule(keyword.clone(), score) {
                            log::error!("Failed to add rule: {}", e);
                            bot.send_message(msg.chat.id, "Failed to add rule.").await?;
                        } else {
                            bot.send_message(msg.chat.id, format!("Added rule: '{}' with score {}", keyword, score)).await?;
                        }
                    } else {
                        bot.send_message(msg.chat.id, "Invalid score.").await?;
                    }
                } else {
                    bot.send_message(msg.chat.id, "Usage: /add_rule <keyword> <score>").await?;
                }
            } else {
                bot.send_message(msg.chat.id, "Only admins can add rules.").await?;
            }
        }
    }
    Ok(())
}

async fn check_message(
    bot: Bot,
    msg: Message,
    rule_manager: Arc<RuleManager>,
) -> Result<(), teloxide::RequestError> {
    if let Some(text) = msg.text() {
        // Skip if the message is a command
        if text.starts_with('/') {
            return Ok(());
        }
        let user_id = msg.from().unwrap().id.to_string();
        let custom_score = rule_manager.check_custom_rules(text);
        let is_spam = custom_score >= 5.0;
        log::info!(
            "Message: '{}', User ID: {}, Custom Score: {}, Is Spam: {}",
            text, user_id, custom_score, is_spam
        );
        if is_spam {
            if let Err(e) = rule_manager.increment_sender_score(&user_id, true) {
                log::error!("Failed to update sender score: {}", e);
            }
            bot.send_message(msg.chat.id, "Spam detected! Admins notified.").await?;
            match notify_admins(&bot, msg.chat.id, text, &rule_manager, &user_id).await {
                Ok(_) => log::info!("Successfully notified admins for spam message: '{}'", text),
                Err(e) => log::error!("Failed to notify admins for spam message '{}': {}", text, e),
            }
        } else {
            if let Err(e) = rule_manager.increment_sender_score(&user_id, false) {
                log::error!("Failed to update sender score: {}", e);
            }
        }
    }
    Ok(())
}


async fn handle_new_chat_members(_bot: Bot, msg: Message) {
    log::info!("Bot added to chat: {:?}", msg.chat);
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let bot = Bot::from_env();
    let rule_manager = Arc::new(RuleManager::new("rules.db").expect("Failed to initialize database"));

    println!("Bot started!");

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint({
                    let rule_manager = rule_manager.clone();
                    move |bot: Bot, msg: Message, cmd: Command| {
                        let rule_manager = rule_manager.clone();
                        async move {
                            answer(bot, msg, cmd, rule_manager).await
                        }
                    }
                }),
        )
        .branch(
            dptree::filter(|msg: Message| msg.text().is_some())
                .endpoint({
                    let rule_manager = rule_manager.clone();
                    move |bot: Bot, msg: Message| {
                        let rule_manager = rule_manager.clone();
                        async move {
                            check_message(bot, msg, rule_manager).await
                        }
                    }
                }),
        )
        .branch(
            dptree::filter(|msg: Message| msg.new_chat_members().is_some() && !msg.new_chat_members().unwrap().is_empty())
                .endpoint(|bot: Bot, msg: Message| async move {
                    handle_new_chat_members(bot, msg).await;
                    Ok(())
                }),
        );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}