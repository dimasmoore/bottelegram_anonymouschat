use redis::AsyncCommands;
use crate::models::{ChatRoom, UserState};
use anyhow::Result;
use uuid::Uuid;
use teloxide::{prelude::*, types::ChatId};

const ROOM_PREFIX: &str = "room:";
const ROOM_LIST_KEY: &str = "rooms";

pub async fn create_room(
    redis: &mut redis::aio::Connection,
    name: String,
    max_members: usize,
) -> Result<ChatRoom> {
    let room_id = Uuid::new_v4().to_string();
    let room = ChatRoom::new(room_id.clone(), name, max_members);
    
    let data = serde_json::to_string(&room)?;
    redis.set(format!("{}{}", ROOM_PREFIX, room_id), data).await?;
    redis.sadd(ROOM_LIST_KEY, room_id).await?;
    
    Ok(room)
}

pub async fn get_room(
    redis: &mut redis::aio::Connection,
    room_id: &str,
) -> Result<Option<ChatRoom>> {
    let data: Option<String> = redis.get(format!("{}{}", ROOM_PREFIX, room_id)).await?;
    Ok(data.map(|d| serde_json::from_str(&d).unwrap()))
}

pub async fn update_room(
    redis: &mut redis::aio::Connection,
    room: &ChatRoom,
) -> Result<()> {
    let data = serde_json::to_string(room)?;
    redis.set(format!("{}{}", ROOM_PREFIX, room.room_id), data).await?;
    Ok(())
}

pub async fn list_rooms(
    redis: &mut redis::aio::Connection,
) -> Result<Vec<ChatRoom>> {
    let room_ids: Vec<String> = redis.smembers(ROOM_LIST_KEY).await?;
    let mut rooms = Vec::new();
    
    for room_id in room_ids {
        if let Some(room) = get_room(redis, &room_id).await? {
            rooms.push(room);
        }
    }
    
    Ok(rooms)
}

pub async fn join_room(
    redis: &mut redis::aio::Connection,
    room_id: &str,
    user_state: &mut UserState,
) -> Result<bool> {
    if let Some(mut room) = get_room(redis, room_id).await? {
        if room.add_member(user_state.chat_id) {
            user_state.current_room = Some(room_id.to_string());
            update_room(redis, &room).await?;
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn leave_room(
    redis: &mut redis::aio::Connection,
    room_id: &str,
    user_state: &mut UserState,
) -> Result<()> {
    if let Some(mut room) = get_room(redis, room_id).await? {
        room.remove_member(user_state.chat_id);
        user_state.current_room = None;
        
        if room.members.is_empty() {
            // Delete empty room
            redis.del(format!("{}{}", ROOM_PREFIX, room_id)).await?;
            redis.srem(ROOM_LIST_KEY, room_id).await?;
        } else {
            update_room(redis, &room).await?;
        }
    }
    Ok(())
}

pub async fn broadcast_to_room(
    bot: &Bot,
    redis: &mut redis::aio::Connection,
    room_id: &str,
    sender_id: i64,
    message: &str,
) -> Result<()> {
    if let Some(room) = get_room(redis, room_id).await? {
        for &member_id in &room.members {
            if member_id != sender_id {
                bot.send_message(
                    ChatId(member_id),
                    format!("👤 Anonymous: {}", message)
                ).await?;
            }
        }
    }
    Ok(())
} 