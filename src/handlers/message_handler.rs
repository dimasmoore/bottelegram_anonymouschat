use teloxide::{
    prelude::*,
    types::{InputFile, MediaKind, MessageKind},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use crate::{
    models::AppState,
    services::{redis_service, content_filter, chat_room},
};

const INACTIVITY_TIMEOUT: u64 = 1800; // 30 minutes in seconds

pub async fn handle_message(
    bot: Bot,
    msg: Message,
    state: Arc<Mutex<AppState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        if text.starts_with('/') {
            return Ok(());
        }
    }

    let chat_id = msg.chat.id.0;
    let state_guard = state.lock().await;
    let mut redis = state_guard.redis.get_async_connection().await.map_err(|e| anyhow::anyhow!(e))?;

    if let Ok(Some(mut current_state)) = redis_service::get_user_state(&mut redis, chat_id).await {
        // Update last activity
        current_state.update_activity();
        redis_service::set_user_state(&mut redis, &current_state).await.map_err(|e| anyhow::anyhow!(e))?;

        // Check for inactivity
        if current_state.is_inactive(INACTIVITY_TIMEOUT) {
            if let Some(partner_id) = current_state.partner_id {
                // Notify partner about disconnection
                bot.send_message(
                    ChatId(partner_id),
                    "⏰ Your chat partner has been disconnected due to inactivity.\n\
                    Use /find to start a new chat!"
                ).await?;

                // Clear partner's state
                if let Ok(Some(mut partner_state)) = redis_service::get_user_state(&mut redis, partner_id).await {
                    partner_state.partner_id = None;
                    partner_state.is_searching = false;
                    redis_service::set_user_state(&mut redis, &partner_state).await.map_err(|e| anyhow::anyhow!(e))?;
                }
            }

            // Clear user's state
            current_state.partner_id = None;
            current_state.is_searching = false;
            redis_service::set_user_state(&mut redis, &current_state).await.map_err(|e| anyhow::anyhow!(e))?;

            bot.send_message(
                msg.chat.id,
                "⏰ You have been disconnected due to inactivity.\n\
                Use /find to start a new chat!"
            ).await?;

            return Ok(());
        }

        // Handle message based on context (private chat or room)
        if let Some(room_id) = &current_state.current_room {
            // Handle room message
            if let Some(text) = msg.text() {
                if content_filter::contains_inappropriate_content(text) {
                    bot.send_message(
                        msg.chat.id,
                        "⚠️ Your message contains inappropriate content and was not sent."
                    ).await?;
                    return Ok(());
                }
                
                let filtered_text = content_filter::filter_message(text);
                chat_room::broadcast_to_room(&bot, &mut redis, room_id, chat_id, &filtered_text).await?;
            }
        } else if let Some(partner_id) = current_state.partner_id {
            // Handle private chat message
            match msg.kind {
                MessageKind::Common(common) => {
                    match common.media_kind {
                        MediaKind::Text(text) => {
                            if content_filter::contains_inappropriate_content(&text.text) {
                                bot.send_message(
                                    msg.chat.id,
                                    "⚠️ Your message contains inappropriate content and was not sent."
                                ).await?;
                                return Ok(());
                            }
                            
                            let filtered_text = content_filter::filter_message(&text.text);
                            bot.send_message(ChatId(partner_id), filtered_text).await?;
                        }
                        MediaKind::Photo(photo) => {
                            if let Some(largest_photo) = photo.photo.last() {
                                let file = InputFile::file_id(&largest_photo.file.id);
                                let caption = photo.caption.unwrap_or_default();
                                if content_filter::contains_inappropriate_content(&caption) {
                                    bot.send_message(
                                        msg.chat.id,
                                        "⚠️ Your photo caption contains inappropriate content and was not sent."
                                    ).await?;
                                    return Ok(());
                                }
                                
                                let filtered_caption = content_filter::filter_message(&caption);
                                bot.send_photo(ChatId(partner_id), file)
                                    .caption(filtered_caption)
                                    .await?;
                            }
                        }
                        MediaKind::Sticker(sticker) => {
                            bot.send_sticker(ChatId(partner_id), InputFile::file_id(&sticker.sticker.file.id))
                                .await?;
                        }
                        MediaKind::Voice(voice) => {
                            let caption = voice.caption.unwrap_or_default();
                            if content_filter::contains_inappropriate_content(&caption) {
                                bot.send_message(
                                    msg.chat.id,
                                    "⚠️ Your voice note caption contains inappropriate content and was not sent."
                                ).await?;
                                return Ok(());
                            }
                            
                            let filtered_caption = content_filter::filter_message(&caption);
                            bot.send_voice(
                                ChatId(partner_id), 
                                InputFile::file_id(&voice.voice.file.id)
                            )
                            .caption(filtered_caption)
                            .await?;
                        }
                        _ => {
                            bot.send_message(
                                msg.chat.id,
                                "❌ This type of message is not supported. You can send text, photos, stickers, or voice notes."
                            ).await?;
                        }
                    }
                }
                _ => {}
            }
        } else {
            bot.send_message(
                msg.chat.id,
                "❌ You're not connected to anyone!\n\
                Use /find to start chatting or /listrooms to join a chat room."
            ).await?;
        }
    }

    Ok(())
} 