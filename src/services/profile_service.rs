use crate::models::MoodEntry;
use anyhow::Result;
use redis::AsyncCommands;
use std::collections::HashMap;
use teloxide::prelude::*;

const MOOD_HISTORY_PREFIX: &str = "mood_history:";
const MOOD_STATS_KEY: &str = "mood_stats";

pub async fn save_mood_history(
    redis: &mut redis::aio::Connection,
    chat_id: i64,
    mood: &MoodEntry,
) -> Result<()> {
    let key = format!("{}{}", MOOD_HISTORY_PREFIX, chat_id);
    let data = serde_json::to_string(mood)?;
    redis.lpush(&key, data).await?;
    // Keep only last 30 days
    redis.ltrim(&key, 0, 29).await?;
    
    // Update mood stats
    let stats_key = MOOD_STATS_KEY;
    let stats: HashMap<String, i32> = match redis.get::<_, String>(stats_key).await {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => HashMap::new(),
    };
    
    let mut new_stats = stats;
    *new_stats.entry(mood.mood.clone()).or_insert(0) += 1;
    redis.set(stats_key, serde_json::to_string(&new_stats)?).await?;
    
    Ok(())
}

pub async fn get_mood_history(
    redis: &mut redis::aio::Connection,
    chat_id: i64,
) -> Result<Vec<MoodEntry>> {
    let key = format!("{}{}", MOOD_HISTORY_PREFIX, chat_id);
    let data: Vec<String> = redis.lrange(&key, 0, 29).await?;
    
    let moods = data
        .into_iter()
        .filter_map(|entry| serde_json::from_str(&entry).ok())
        .collect();
    
    Ok(moods)
}

pub async fn get_mood_stats(
    redis: &mut redis::aio::Connection,
) -> Result<HashMap<String, i32>> {
    let stats: Option<String> = redis.get(MOOD_STATS_KEY).await?;
    Ok(stats
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default())
}

pub async fn broadcast_message(
    bot: &Bot,
    redis: &mut redis::aio::Connection,
    message: &str,
    sender_id: i64,
) -> Result<usize> {
    let pattern = "user:*";
    let keys: Vec<String> = redis.keys(pattern).await?;
    let mut sent_count = 0;
    
    for key in keys {
        if let Ok(Some(state)) = super::redis_service::get_user_state(redis, key.split(':').nth(1).unwrap().parse()?).await {
            if state.chat_id != sender_id {
                if bot
                    .send_message(
                        ChatId(state.chat_id),
                        format!("📢 Broadcast Message:\n\n{}", message),
                    )
                    .await
                    .is_ok()
                {
                    sent_count += 1;
                }
            }
        }
    }
    
    Ok(sent_count)
} 