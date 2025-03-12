use teloxide::{prelude::*, utils::command::BotCommands};
use dotenv::dotenv;
use std::sync::Arc;
use spam_bot_mvp::rules::RuleManager;
use spam_bot_mvp::utils::{is_admin, notify_admins};

/// The main entry point for the Telegram spam detection bot.
///
/// This module sets up a Telegram bot using the `teloxide` framework to detect spam messages
/// in chats, manage custom spam rules, and notify administrators. It integrates with the
/// `rules` module for spam detection logic and the `utils` module for admin-related utilities.
///
/// The bot supports the following features:
/// - Responds to commands (`/start`, `/report`, `/add_rule`) for bot interaction.
/// - Automatically checks incoming messages for spam using custom Lua rules.
/// - Notifies admins when spam is detected, with a fallback to group notifications.
/// - Logs bot activity and errors using the `log` crate and `env_logger`.
///
/// # Dependencies
/// - `teloxide`: For Telegram bot API interactions.
/// - `dotenv`: For loading environment variables (e.g., bot token).
/// - `std::sync::Arc`: For thread-safe sharing of the `RuleManager`.
/// - `spam_bot_mvp::rules`: For spam detection and rule management.
/// - `spam_bot_mvp::utils`: For admin checks and notifications.
///
/// # Environment Variables
/// - `TELOXIDE_TOKEN`: The Telegram bot token, loaded from a `.env` file or environment.
///
/// # Examples
/// To run the bot:
/// 1. Create a `.env` file with `TELOXIDE_TOKEN=your_bot_token`.
/// 2. Ensure a `rules.db` SQLite database exists (created automatically if not).
/// 3. Run the bot with `cargo run`.
///
/// The bot will respond to commands in Telegram chats and detect spam messages.
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Bot commands")]
enum Command {
    /// Starts the bot and sends a welcome message.
    #[command(description = "Start the bot")]
    Start,

    /// Reports a message as spam by replying to it.
    #[command(description = "Report a message as spam")]
    Report,

    /// Adds a custom spam rule (admin only).
    ///
    /// Format: `/add_rule <keyword> <score>`.
    /// Example: `/add_rule spam 10.0` adds a rule to flag "spam" with a score of 10.0.
    #[command(description = "Add a custom spam rule (admin only, format: /add_rule <keyword> <score>)")]
    AddRule(String),
}

/// Handles bot commands (`/start`, `/report`, `/add_rule`).
///
/// Processes incoming commands, performs the associated actions, and sends responses
/// to the chat. Only admins can use `/add_rule` to add custom spam rules.
///
/// # Arguments
/// * `bot` - The Telegram bot instance.
/// * `msg` - The message containing the command.
/// * `cmd` - The parsed command (e.g., `Start`, `Report`, `AddRule`).
/// * `rule_manager` - A thread-safe reference to the `RuleManager` for rule operations.
///
/// # Returns
/// * `Result<()>` - A `Result` indicating success or a `teloxide::RequestError` if the operation fails.
///
/// # Panics
/// * Panics if `msg.from()` is `None` in contexts where the sender is required.
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

/// Checks incoming messages for spam and notifies admins if detected.
///
/// Evaluates each text message against custom spam rules. If a message is flagged as spam
/// (score >= 5.0), it increments the sender’s spam score, sends a notification to the chat,
/// and attempts to notify admins. Non-spam messages increment the sender’s message count
/// without affecting the spam score.
///
/// # Arguments
/// * `bot` - The Telegram bot instance.
/// * `msg` - The incoming message to check.
/// * `rule_manager` - A thread-safe reference to the `RuleManager` for rule operations.
///
/// # Returns
/// * `Result<()>` - A `Result` indicating success or a `teloxide::RequestError` if the operation fails.
///
/// # Panics
/// * Panics if `msg.from()` is `None` (i.e., no sender information).
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

/// Logs when the bot is added to a new chat.
///
/// This function is triggered when the bot is added to a group or channel.
/// It logs the chat details for debugging purposes but does not send any messages.
///
/// # Arguments
/// * `bot` - The Telegram bot instance (unused in this function).
/// * `msg` - The message indicating the bot was added to a chat.
async fn handle_new_chat_members(_bot: Bot, msg: Message) {
    log::info!("Bot added to chat: {:?}", msg.chat);
}

/// The main entry point for the bot application.
///
/// Initializes the bot, sets up the `RuleManager`, and starts the event dispatcher.
/// The bot listens for:
/// - Commands (`/start`, `/report`, `/add_rule`) via the `answer` handler.
/// - Text messages to check for spam via the `check_message` handler.
/// - Events when the bot is added to a chat via the `handle_new_chat_members` handler.
///
/// # Panics
/// * Panics if the `RuleManager` cannot be initialized (e.g., database failure).
/// * Panics if the `TELOXIDE_TOKEN` environment variable is not set.
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