use anyhow::Result;
use teloxide::macros::BotCommands;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::types::ParseMode;
use teloxide::Bot;
use tracing::info;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum EnhancerCommands {
    Start,
}

pub(crate) async fn command_handler(bot: Bot, msg: Message, cmd: EnhancerCommands) -> Result<()> {
    info!("fn command_handler: got command");
    let user_id = msg.from.as_ref().map(|user| user.id.0).unwrap_or(0);

    match cmd {
        EnhancerCommands::Start => {
            bot.send_message(msg.chat.id, "Hello!")
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }
    Ok(())
}
