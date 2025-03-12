use teloxide::{Bot, types::{Message, ChatId}};
use teloxide::errors::RequestError;
use teloxide::prelude::Requester;
use crate::rules::RuleManager;

pub async fn is_admin(bot: &Bot, msg: &Message) -> Result<bool, RequestError> {
    if msg.chat.is_private() {
        Ok(true)
    } else {
        let admins = bot.get_chat_administrators(msg.chat.id).await?;
        let user_id = msg.from().unwrap().id;
        log::info!("Checking admin status for user {} in chat {}", user_id, msg.chat.id);
        let is_admin = admins.iter().any(|admin| {
            log::info!("Admin found: {}", admin.user.id);
            admin.user.id == user_id
        });
        Ok(is_admin)
    }
}

pub async fn notify_admins(bot: &Bot, chat_id: ChatId, text: &str, rule_manager: &RuleManager, user_id: &str) -> Result<(), RequestError> {
    let spam_score = rule_manager.get_sender_score(user_id);
    let message = format!("Spam detected: {}\nSender ID: {}\nSpam Score: {}", text, user_id, spam_score);
    log::info!("Attempting to notify admins in chat {}", chat_id);
    if chat_id.is_group() {
        let admins_result = bot.get_chat_administrators(chat_id).await;
        match admins_result {
            Ok(admins) => {
                log::info!("Found {} admins: {:?}", admins.len(), admins.iter().map(|a| a.user.id).collect::<Vec<_>>());
                if admins.is_empty() {
                    log::warn!("No admins found in chat {}. Sending fallback notification in group.", chat_id);
                    bot.send_message(chat_id, &message).await?;
                } else {
                    for admin in admins {
                        let admin_user_id = admin.user.id;
                        log::info!("Attempting to notify admin {}", admin_user_id);
                        match bot.send_message(admin_user_id, &message).await {
                            Ok(_) => log::info!("Notification sent to admin {}", admin_user_id),
                            Err(e) => log::error!("Failed to send notification to admin {}: {}", admin_user_id, e),
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