mod commands;
mod handlers;
mod models;
mod services;

use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use teloxide::prelude::*;
use anyhow::Result;

use crate::models::AppState;
use crate::commands::Command;
use crate::handlers::{command_handler, message_handler};
use crate::services::mongodb_service::MongoDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("🚀 Starting Anonymous Chat Bot...");
    
    dotenvy::dotenv().ok();
    
    let redis_client = redis::Client::open(
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
    ).map_err(|e| anyhow::anyhow!(e))?;

    let mongodb = MongoDB::new().await?;
    
    let state = Arc::new(Mutex::new(AppState {
        redis: redis_client,
        mongodb,
    }));

    let bot = Bot::from_env();
    let state_clone = state.clone();

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(move |bot: Bot, msg: Message, cmd: Command| {
                    command_handler::handle_command(bot, msg, cmd, state.clone())
                }),
        )
        .branch(
            dptree::filter(|msg: Message| !msg.text().map(|text| text.starts_with('/')).unwrap_or(false))
                .endpoint(move |bot: Bot, msg: Message| {
                    message_handler::handle_message(bot, msg, state_clone.clone())
                }),
        );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
