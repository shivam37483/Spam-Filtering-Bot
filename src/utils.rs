use crate::rules::RuleManager;
use teloxide::errors::RequestError;
use teloxide::prelude::Requester;
/// A module providing utility functions for Telegram bot administration and notifications.
///
/// This module contains helper functions to check user admin status and notify administrators
/// about spam detection events. It leverages the `teloxide` library for Telegram interactions
/// and integrates with the `rules` module for spam score management.
use teloxide::{
    types::{ChatId, Message},
    Bot,
};

/// Checks if a user is an administrator in the given chat.
///
/// Determines whether the sender of a message is an admin. In private chats,
/// all users are considered admins by default. In group chats, it queries the
/// Telegram API to fetch the list of administrators and checks if the user's
/// ID is included.
///
/// # Arguments
/// * `bot` - A reference to the Telegram bot instance.
/// * `msg` - A reference to the message containing the user and chat context.
///
/// # Returns
/// * `Result<bool>` - A `Result` containing `true` if the user is an admin,
///   `false` otherwise, or a `RequestError` if the API call fails.
///
/// # Panics
/// * Panics if `msg.from()` is `None` (i.e., no sender information).
pub async fn is_admin(bot: &Bot, msg: &Message) -> Result<bool, RequestError> {
    if msg.chat.is_private() {
        Ok(true)
    } else {
        let admins = bot.get_chat_administrators(msg.chat.id).await?;
        let user_id = msg.from().unwrap().id;
        log::info!(
            "Checking admin status for user {} in chat {}",
            user_id,
            msg.chat.id
        );
        let is_admin = admins.iter().any(|admin| {
            log::info!("Admin found: {}", admin.user.id);
            admin.user.id == user_id
        });
        Ok(is_admin)
    }
}

/// Notifies administrators about a detected spam message.
///
/// Attempts to send a notification to all admins in a group chat with details
/// of the spam message, including the text, sender ID, and spam score. In private
/// chats, the notification is sent to the same chat. If fetching admins fails
/// or no admins are found, a fallback notification is sent in the group chat.
///
/// # Arguments
/// * `bot` - A reference to the Telegram bot instance.
/// * `chat_id` - The ID of the chat where the spam was detected.
/// * `text` - The text of the spam message.
/// * `rule_manager` - A reference to the `RuleManager` for retrieving sender scores.
/// * `user_id` - The ID of the sender of the spam message.
///
/// # Returns
/// * `Result<()>` - A `Result` indicating success or a `RequestError` if
///   sending the notification fails.
///
/// # Notes
/// * Logs the notification process and any errors for debugging.
/// * Uses a fallback mechanism to ensure visibility if private notifications fail.
pub async fn notify_admins(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
    rule_manager: &RuleManager,
    user_id: &str,
) -> Result<(), RequestError> {
    let spam_score = rule_manager.get_sender_score(user_id);
    let message = format!(
        "Spam detected: {}\nSender ID: {}\nSpam Score: {}",
        text, user_id, spam_score
    );
    log::info!("Attempting to notify admins in chat {}", chat_id);
    if chat_id.is_group() {
        let admins_result = bot.get_chat_administrators(chat_id).await;
        match admins_result {
            Ok(admins) => {
                log::info!(
                    "Found {} admins: {:?}",
                    admins.len(),
                    admins.iter().map(|a| a.user.id).collect::<Vec<_>>()
                );
                if admins.is_empty() {
                    log::warn!(
                        "No admins found in chat {}. Sending fallback notification in group.",
                        chat_id
                    );
                    bot.send_message(chat_id, &message).await?;
                } else {
                    for admin in admins {
                        let admin_user_id = admin.user.id;
                        log::info!("Attempting to notify admin {}", admin_user_id);
                        match bot.send_message(admin_user_id, &message).await {
                            Ok(_) => log::info!("Notification sent to admin {}", admin_user_id),
                            Err(e) => log::error!(
                                "Failed to send notification to admin {}: {}",
                                admin_user_id,
                                e
                            ),
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to fetch admins for chat {}: {}. Sending fallback notification in group.", chat_id, e);
                bot.send_message(chat_id, &message).await?;
            }
        }
    } else {
        bot.send_message(chat_id, message).await?;
    }
    Ok(())
}
