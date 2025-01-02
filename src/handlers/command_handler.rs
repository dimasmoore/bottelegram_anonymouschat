use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use crate::{
    models::{AppState, UserState},
    commands::Command,
    services::{redis_service, chat_room, profile_service},
};

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: Arc<Mutex<AppState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let state_guard = state.lock().await;
    let mut redis = state_guard.redis.get_async_connection().await.map_err(|e| anyhow::anyhow!(e))?;
    
    log::info!("📝 Received command: {:?} from user {}", cmd, chat_id);
    
    match cmd {
        Command::Help => {
            bot.send_message(
                msg.chat.id,
                format!("🌟 Welcome to Anonymous Chat! 🌟\n\n{}", Command::descriptions().to_string())
            ).await?;
        }
        Command::Start => {
            log::info!("🎉 New user starting bot: {}", chat_id);
            
            // Clean up old data if exists
            redis_service::cleanup_user_state(&mut redis, chat_id).await.map_err(|e| anyhow::anyhow!(e))?;
            
            // Create new user state
            let new_state = UserState::new(chat_id);
            redis_service::set_user_state(&mut redis, &new_state).await.map_err(|e| anyhow::anyhow!(e))?;
            
            bot.send_message(
                msg.chat.id,
                "🎈 Welcome to Anonymous Chat Bot! 🎈\n\n\
                Here you can chat anonymously with random people or join chat rooms!\n\n\
                📝 Commands:\n\
                /find - Find a random chat partner\n\
                /createroom - Create a new chat room\n\
                /listrooms - See available chat rooms\n\
                /joinroom - Join a chat room\n\
                /leave - Leave current chat or room\n\
                /setprofile - Set your profile\n\
                /viewprofile - View your profile\n\
                /help - Show all commands\n\n\
                📱 Supported messages:\n\
                • Text messages 💬\n\
                • Photos 📸\n\
                • Stickers 🎯\n\
                • Voice Notes 🎤\n\n\
                🔒 Your privacy is our priority! Stay safe and have fun!"
            ).await?;
            
            log::info!("✅ User {} initialized successfully", chat_id);
        }
        Command::Find => {
            // Check if user is in a room
            if let Ok(Some(current_state)) = redis_service::get_user_state(&mut redis, chat_id).await {
                if current_state.current_room.is_some() {
                    bot.send_message(
                        msg.chat.id,
                        "❌ You're currently in a chat room! Use /leave first to find a private chat partner."
                    ).await?;
                    return Ok(());
                }
                if current_state.partner_id.is_some() {
                    bot.send_message(
                        msg.chat.id,
                        "❌ You're already in a chat! Use /leave first to find a new partner."
                    ).await?;
                    return Ok(());
                }
            }

            // Set user as searching
            let user_state = UserState::new(chat_id);
            let mut searching_state = user_state.clone();
            searching_state.is_searching = true;
            redis_service::set_user_state(&mut redis, &searching_state).await.map_err(|e| anyhow::anyhow!(e))?;

            // Try to find a partner
            if let Some(partner_id) = redis_service::find_random_partner(&mut redis, chat_id).await.map_err(|e| anyhow::anyhow!(e))? {
                // Match found!
                redis_service::connect_users(&mut redis, chat_id, partner_id).await.map_err(|e| anyhow::anyhow!(e))?;
                
                let match_message = "🎉 Chat partner found! Say hi! 👋\n\
                    You can send:\n\
                    • Text messages 💬\n\
                    • Photos 📸\n\
                    • Stickers 🎯\n\
                    • Voice Notes 🎤\n\n\
                    Use /leave when you want to end the chat.";
                
                bot.send_message(ChatId(chat_id), match_message).await?;
                bot.send_message(ChatId(partner_id), match_message).await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    "🔍 Looking for a chat partner... Please wait!"
                ).await?;
            }
        }
        Command::CreateRoom { name, max_members } => {
            if let Ok(Some(current_state)) = redis_service::get_user_state(&mut redis, chat_id).await {
                if current_state.partner_id.is_some() || current_state.current_room.is_some() {
                    bot.send_message(
                        msg.chat.id,
                        "❌ You must leave your current chat or room first!"
                    ).await?;
                    return Ok(());
                }
            }

            // Parse max_members from string to number
            let max_members = match max_members.parse::<usize>() {
                Ok(num) => num.min(50).max(2), // Limit room size between 2 and 50
                Err(_) => {
                    bot.send_message(
                        msg.chat.id,
                        "❌ Invalid number for max_members. Please use a number between 2 and 50.\n\
                        Example: /createroom FunChat 10"
                    ).await?;
                    return Ok(());
                }
            };

            let room = chat_room::create_room(&mut redis, name.clone(), max_members).await?;
            
            bot.send_message(
                msg.chat.id,
                format!("🎉 Chat room '{}' created!\n\
                    Room ID: {}\n\
                    Maximum members: {}\n\n\
                    Share this Room ID with others to let them join using /joinroom command!",
                    name, room.room_id, max_members
                )
            ).await?;
        }
        Command::ListRooms => {
            let rooms = chat_room::list_rooms(&mut redis).await?;
            if rooms.is_empty() {
                bot.send_message(
                    msg.chat.id,
                    "😔 No active chat rooms found.\n\
                    Create one using /createroom command!"
                ).await?;
            } else {
                let mut message = "📋 Available Chat Rooms:\n\n".to_string();
                for room in rooms {
                    message.push_str(&format!(
                        "🏠 Name: {}\n\
                        📝 ID: {}\n\
                        👥 Members: {}/{}\n\n",
                        room.name,
                        room.room_id,
                        room.members.len(),
                        room.max_members
                    ));
                }
                message.push_str("\nUse /joinroom command with a room ID to join!");
                bot.send_message(msg.chat.id, message).await?;
            }
        }
        Command::JoinRoom(room_id) => {
            if let Ok(Some(mut current_state)) = redis_service::get_user_state(&mut redis, chat_id).await {
                if current_state.partner_id.is_some() || current_state.current_room.is_some() {
                    bot.send_message(
                        msg.chat.id,
                        "❌ You must leave your current chat or room first!"
                    ).await?;
                    return Ok(());
                }

                if chat_room::join_room(&mut redis, &room_id, &mut current_state).await? {
                    redis_service::set_user_state(&mut redis, &current_state).await.map_err(|e| anyhow::anyhow!(e))?;
                    
                    if let Some(room) = chat_room::get_room(&mut redis, &room_id).await? {
                        bot.send_message(
                            msg.chat.id,
                            format!("🎉 Welcome to chat room '{}'!\n\
                                👥 Current members: {}/{}\n\n\
                                Start chatting or use /leave to exit the room.",
                                room.name,
                                room.members.len(),
                                room.max_members
                            )
                        ).await?;

                        // Notify other room members
                        for &member_id in &room.members {
                            if member_id != chat_id {
                                bot.send_message(
                                    ChatId(member_id),
                                    "👋 A new user has joined the chat room!"
                                ).await?;
                            }
                        }
                    }
                } else {
                    bot.send_message(
                        msg.chat.id,
                        "❌ Could not join the room. It might be full or no longer exists."
                    ).await?;
                }
            }
        }
        Command::Leave => {
            if let Ok(Some(mut current_state)) = redis_service::get_user_state(&mut redis, chat_id).await {
                if let Some(room_id) = current_state.current_room.clone() {
                    // Leave chat room
                    chat_room::leave_room(&mut redis, &room_id, &mut current_state).await?;
                    redis_service::set_user_state(&mut redis, &current_state).await.map_err(|e| anyhow::anyhow!(e))?;
                    
                    bot.send_message(
                        msg.chat.id,
                        "👋 You've left the chat room.\n\
                        Use /find to start a private chat or /listrooms to see available rooms!"
                    ).await?;
                } else if let Some(partner_id) = current_state.partner_id {
                    // Leave private chat
                    bot.send_message(
                        ChatId(partner_id),
                        "👋 Your chat partner has left the chat.\n\
                        Use /find to start a new chat!"
                    ).await?;
                    
                    // Clear partner's state
                    if let Ok(Some(mut partner_state)) = redis_service::get_user_state(&mut redis, partner_id).await {
                        partner_state.partner_id = None;
                        partner_state.is_searching = false;
                        redis_service::set_user_state(&mut redis, &partner_state).await.map_err(|e| anyhow::anyhow!(e))?;
                    }
                    
                    // Clear user's state
                    current_state.partner_id = None;
                    current_state.is_searching = false;
                    redis_service::set_user_state(&mut redis, &current_state).await.map_err(|e| anyhow::anyhow!(e))?;
                    
                    bot.send_message(
                        msg.chat.id,
                        "👋 You've left the chat.\n\
                        Use /find to start a new chat!"
                    ).await?;
                } else {
                    bot.send_message(
                        msg.chat.id,
                        "❌ You're not in a chat or room!\n\
                        Use /find to start chatting or /listrooms to see available rooms."
                    ).await?;
                }
            }
        }
        Command::SetProfile { nickname, emoji, bio } => {
            log::info!("🔄 Processing /setprofile command for user {}", chat_id);
            log::info!("👤 Setting profile for user {}: {} {} {}", chat_id, nickname, emoji, bio);
            
            // Save to Redis for session data
            if let Ok(Some(mut current_state)) = redis_service::get_user_state(&mut redis, chat_id).await {
                current_state.set_profile(nickname.clone(), emoji.clone(), bio.clone());
                if let Err(e) = redis_service::set_user_state(&mut redis, &current_state).await {
                    log::error!("❌ Failed to save profile to Redis for user {}: {}", chat_id, e);
                    bot.send_message(
                        msg.chat.id,
                        "❌ Failed to save profile. Please try again later."
                    ).await?;
                    return Ok(());
                }
                log::info!("✅ Profile saved to Redis for user {}", chat_id);
            } else {
                log::warn!("⚠️ No existing state found for user {}, creating new", chat_id);
                let mut new_state = UserState::new(chat_id);
                new_state.set_profile(nickname.clone(), emoji.clone(), bio.clone());
                redis_service::set_user_state(&mut redis, &new_state).await?;
            }

            // Save to MongoDB for persistence
            if let Ok(Some(current_state)) = redis_service::get_user_state(&mut redis, chat_id).await {
                if let Some(profile) = current_state.profile.clone() {
                    match state_guard.mongodb.save_profile(chat_id, profile.clone()).await {
                        Ok(_) => {
                            log::info!("✅ Profile saved to MongoDB for user {}", chat_id);
                            bot.send_message(
                                msg.chat.id,
                                format!("✅ Profile updated successfully!\n\n\
                                    Nickname: {}\n\
                                    Avatar: {}\n\
                                    Bio: {}", 
                                    profile.nickname, profile.avatar_emoji, profile.bio
                                )
                            ).await?;
                        },
                        Err(e) => {
                            log::error!("❌ Failed to save profile to MongoDB for user {}: {}", chat_id, e);
                            bot.send_message(
                                msg.chat.id,
                                "⚠️ Profile saved for this session but might not persist. Please try again later."
                            ).await?;
                        }
                    }
                }
            }
        }
        Command::ViewProfile => {
            log::info!("🔄 Processing /viewprofile command for user {}", chat_id);
            
            let state_guard = state.lock().await;
            match state_guard.mongodb.get_profile(chat_id).await {
                Ok(Some(profile)) => {
                    log::info!("✅ Retrieved profile for user {}", chat_id);
                    bot.send_message(
                        msg.chat.id,
                        format!("👤 Your Profile:\n\n\
                            Nickname: {}\n\
                            Avatar: {}\n\
                            Bio: {}\n\
                            Created: {}\n\
                            Last Updated: {}", 
                            profile.nickname,
                            profile.avatar_emoji,
                            profile.bio,
                            profile.created_at.format("%Y-%m-%d %H:%M:%S"),
                            profile.updated_at.format("%Y-%m-%d %H:%M:%S")
                        )
                    ).await?;
                },
                Ok(None) => {
                    log::info!("ℹ️ No profile found for user {}", chat_id);
                    bot.send_message(
                        msg.chat.id,
                        "❌ You haven't set up your profile yet!\n\
                        Use /setprofile <nickname> <emoji> <bio> to create one."
                    ).await?;
                },
                Err(e) => {
                    log::error!("❌ Failed to retrieve profile for user {}: {}", chat_id, e);
                    bot.send_message(
                        msg.chat.id,
                        "❌ Failed to retrieve profile. Please try again later."
                    ).await?;
                }
            }
        }
        Command::SetMood { mood, note } => {
            if let Ok(Some(mut current_state)) = redis_service::get_user_state(&mut redis, chat_id).await {
                current_state.set_mood(mood.clone(), Some(note.clone()));
                redis_service::set_user_state(&mut redis, &current_state).await.map_err(|e| anyhow::anyhow!(e))?;
                
                if let Some(mood_entry) = &current_state.daily_mood {
                    profile_service::save_mood_history(&mut redis, chat_id, mood_entry).await?;
                }
                
                bot.send_message(
                    msg.chat.id,
                    format!("✅ Mood updated successfully!\n\nCurrent mood: {}\nNote: {}", 
                        mood, note
                    )
                ).await?;
            }
        }
        Command::ViewMood => {
            if let Ok(moods) = profile_service::get_mood_history(&mut redis, chat_id).await {
                if moods.is_empty() {
                    bot.send_message(
                        msg.chat.id,
                        "📊 You haven't recorded any moods yet!\n\
                        Use /setmood <mood> [note] to start tracking."
                    ).await?;
                } else {
                    let mut message = "📊 Your Mood History:\n\n".to_string();
                    for (i, mood) in moods.iter().enumerate() {
                        let note = mood.note.as_ref().map(|n| format!("\nNote: {}", n)).unwrap_or_default();
                        message.push_str(&format!(
                            "{}. Mood: {}{}\n\n",
                            i + 1,
                            mood.mood,
                            note
                        ));
                    }
                    bot.send_message(msg.chat.id, message).await?;
                }
            }
        }
        Command::MoodStats => {
            if let Ok(stats) = profile_service::get_mood_stats(&mut redis).await {
                if stats.is_empty() {
                    bot.send_message(
                        msg.chat.id,
                        "📊 No mood data available yet!"
                    ).await?;
                } else {
                    let mut message = "📊 Anonymous Mood Statistics:\n\n".to_string();
                    for (mood, count) in stats {
                        message.push_str(&format!("{}: {} times\n", mood, count));
                    }
                    bot.send_message(msg.chat.id, message).await?;
                }
            }
        }
        Command::Broadcast(message) => {
            if let Ok(Some(current_state)) = redis_service::get_user_state(&mut redis, chat_id).await {
                if !current_state.is_admin {
                    bot.send_message(
                        msg.chat.id,
                        "❌ This command is only available for administrators."
                    ).await?;
                    return Ok(());
                }
                
                let sent_count = profile_service::broadcast_message(&bot, &mut redis, &message, chat_id).await?;
                bot.send_message(
                    msg.chat.id,
                    format!("✅ Broadcast message sent to {} users.", sent_count)
                ).await?;
            }
        }
    }
    Ok(())
} 