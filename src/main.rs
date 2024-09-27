mod ai_utils;
mod command_handler;
mod message_handler;
mod parser;

use crate::command_handler::{command_handler, EnhancerCommands};
use crate::message_handler::{message_handler, AppState};
use anyhow::Result;
use dotenv::dotenv;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let stdout_layer = fmt::layer().with_ansi(true);
    let env_filter = EnvFilter::new("info");

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .init();

    info!("Starting LLM Enhancer...");

    let bot = Bot::from_env();

    let app_state = Arc::new(AppState {
        user_state: Mutex::new(HashMap::new()),
    });

    let cmd_handler = Update::filter_message()
        .filter_command::<EnhancerCommands>()
        .endpoint(command_handler);

    let chat_handler = Update::filter_message().endpoint(message_handler);

    let handler = dptree::entry().branch(cmd_handler).branch(chat_handler);

    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            teloxide::update_listeners::polling_default(bot).await,
            LoggingErrorHandler::with_custom_text("Dispatcher: an error from the update listener"),
        )
        .await;

    Ok(())
}
