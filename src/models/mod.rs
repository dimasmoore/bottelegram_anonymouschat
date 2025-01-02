use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub nickname: String,
    pub avatar_emoji: String,
    pub bio: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoodEntry {
    pub timestamp: DateTime<Utc>,
    pub mood: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserState {
    pub chat_id: i64,
    pub partner_id: Option<i64>,
    pub is_searching: bool,
    pub last_activity: u64,
    pub current_room: Option<String>,
    pub profile: Option<UserProfile>,
    pub is_admin: bool,
    pub daily_mood: Option<MoodEntry>,
}

impl UserState {
    pub fn new(chat_id: i64) -> Self {
        Self {
            chat_id,
            partner_id: None,
            is_searching: false,
            last_activity: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            current_room: None,
            profile: None,
            is_admin: false,
            daily_mood: None,
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    pub fn is_inactive(&self, timeout_secs: u64) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_time - self.last_activity > timeout_secs
    }

    pub fn set_profile(&mut self, nickname: String, avatar_emoji: String, bio: String) {
        let now = Utc::now();
        self.profile = Some(UserProfile {
            nickname,
            avatar_emoji,
            bio,
            created_at: now,
            updated_at: now,
        });
    }

    pub fn set_mood(&mut self, mood: String, note: Option<String>) {
        self.daily_mood = Some(MoodEntry {
            timestamp: Utc::now(),
            mood,
            note,
        });
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatRoom {
    pub room_id: String,
    pub name: String,
    pub members: HashSet<i64>,
    pub max_members: usize,
}

impl ChatRoom {
    pub fn new(room_id: String, name: String, max_members: usize) -> Self {
        Self {
            room_id,
            name,
            members: HashSet::new(),
            max_members,
        }
    }

    pub fn can_join(&self) -> bool {
        self.members.len() < self.max_members
    }

    pub fn add_member(&mut self, chat_id: i64) -> bool {
        if self.can_join() {
            self.members.insert(chat_id);
            true
        } else {
            false
        }
    }

    pub fn remove_member(&mut self, chat_id: i64) {
        self.members.remove(&chat_id);
    }
}

pub struct AppState {
    pub redis: redis::Client,
    pub mongodb: crate::services::mongodb_service::MongoDB,
} 