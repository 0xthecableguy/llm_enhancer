use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::types::ParseMode;
use tracing::info;

pub(crate) async fn message_handler(
    bot: Bot,
    msg: Message,
) -> anyhow::Result<()> {
    info!("fn message_handler: got message");

    bot.send_message(msg.chat.id, "Hello!")
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}