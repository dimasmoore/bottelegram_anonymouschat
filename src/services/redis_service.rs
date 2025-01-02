use redis::AsyncCommands;
use crate::models::UserState;
use anyhow::{Result, anyhow};
use rand::seq::SliceRandom;

pub async fn get_user_state(
    redis: &mut redis::aio::Connection,
    chat_id: i64,
) -> Result<Option<UserState>> {
    let key = format!("user:{}", chat_id);
    let data: Option<String> = redis.get(&key).await?;
    
    match data {
        Some(json_str) => {
            match serde_json::from_str(&json_str) {
                Ok(state) => Ok(Some(state)),
                Err(_) => {
                    // Handle old data format by creating new state
                    let new_state = UserState::new(chat_id);
                    set_user_state(redis, &new_state).await?;
                    Ok(Some(new_state))
                }
            }
        }
        None => Ok(None)
    }
}

pub async fn set_user_state(
    redis: &mut redis::aio::Connection,
    state: &UserState,
) -> Result<()> {
    let key = format!("user:{}", state.chat_id);
    let data = serde_json::to_string(state).map_err(|e| anyhow!("Serialization error: {}", e))?;
    redis.set(&key, data).await?;
    Ok(())
}

pub async fn find_random_partner(
    redis: &mut redis::aio::Connection,
    chat_id: i64,
) -> Result<Option<i64>> {
    let pattern = "user:*";
    let keys: Vec<String> = redis.keys(pattern).await?;
    
    let mut available_partners = Vec::new();
    
    for key in keys {
        if let Ok(Some(state)) = get_user_state(redis, key.split(':').nth(1).unwrap().parse()?).await {
            if state.chat_id != chat_id && state.is_searching && state.partner_id.is_none() {
                available_partners.push(state.chat_id);
            }
        }
    }
    
    Ok(available_partners.choose(&mut rand::thread_rng()).copied())
}

pub async fn connect_users(
    redis: &mut redis::aio::Connection,
    user1_id: i64,
    user2_id: i64,
) -> Result<()> {
    // Update user1 state
    let mut user1_state = match get_user_state(redis, user1_id).await? {
        Some(state) => state,
        None => UserState::new(user1_id),
    };
    user1_state.partner_id = Some(user2_id);
    user1_state.is_searching = false;
    user1_state.update_activity();
    set_user_state(redis, &user1_state).await?;
    
    // Update user2 state
    let mut user2_state = match get_user_state(redis, user2_id).await? {
        Some(state) => state,
        None => UserState::new(user2_id),
    };
    user2_state.partner_id = Some(user1_id);
    user2_state.is_searching = false;
    user2_state.update_activity();
    set_user_state(redis, &user2_state).await?;
    
    Ok(())
}

// Helper function to clean up old data
pub async fn cleanup_user_state(
    redis: &mut redis::aio::Connection,
    chat_id: i64,
) -> Result<()> {
    let key = format!("user:{}", chat_id);
    redis.del(&key).await?;
    Ok(())
} 