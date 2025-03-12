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
        Ok(admins.iter().any(|admin| admin.user.id == user_id))
    }
}

pub async fn notify_admins(bot: &Bot, chat_id: ChatId, text: &str, rule_manager: &RuleManager, user_id: &str) -> Result<(), RequestError> {
    let spam_score = rule_manager.get_sender_score(user_id);
    let message = format!("Spam detected: {}\nSender ID: {}\nSpam Score: {}", text, user_id, spam_score);
    if chat_id.is_group() {
        let admins = bot.get_chat_administrators(chat_id).await?;
        for admin in admins {
            bot.send_message(admin.user.id, message.clone()).await?;
        }
    } else {
        bot.send_message(chat_id, message).await?;
    }
    Ok(())
}