use teloxide::{prelude::*, utils::command::BotCommands};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let bot = Bot::from_env();

    println!("Bot started!");

    Dispatcher::builder(bot, Update::filter_message().branch(
        dptree::entry().filter_command::<Command>().endpoint(answer),
    ))
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Bot commands")]
enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Report a message as spam")]
    Report,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::Start => {
            bot.send_message(msg.chat.id, "Hello! I'm a spam filter bot.").await?;
        }
        Command::Report => {
            if let Some(reply) = msg.reply_to_message() {
                // For non-textual messages
                let text = reply.text().unwrap_or("(non-text message)");
                bot.send_message(msg.chat.id, format!("Reported: {}", text)).await?;
            } else {
                bot.send_message(msg.chat.id, "Please reply to a message to report it.").await?;
            }
        }
    }
    Ok(())
}