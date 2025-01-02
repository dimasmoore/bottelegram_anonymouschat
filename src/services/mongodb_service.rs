use mongodb::{Client, Collection, Database, IndexModel, options::IndexOptions};
use crate::models::{UserProfile, MoodEntry};
use anyhow::Result;
use futures::StreamExt;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const DB_NAME: &str = "telegram_anonymous_chat";
const USERS_COLLECTION: &str = "users";

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDocument {
    pub chat_id: i64,
    pub profile: Option<UserProfile>,
    pub moods: Vec<MoodEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct MongoDB {
    db: Database,
}

impl MongoDB {
    pub async fn new() -> Result<Self> {
        let uri = std::env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        
        log::info!("🗄️ Connecting to MongoDB at: {}", uri);
        
        let client = Client::with_uri_str(&uri).await?;
        let db = client.database(DB_NAME);
        
        // Ensure indexes exist
        let users = db.collection::<UserDocument>(USERS_COLLECTION);
        
        let index = IndexModel::builder()
            .keys(mongodb::bson::doc! { "chat_id": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
            
        users.create_index(index, None).await?;

        log::info!("✅ Successfully connected to MongoDB database: {}", DB_NAME);
        
        Ok(Self { db })
    }

    fn users_collection(&self) -> Collection<UserDocument> {
        self.db.collection(USERS_COLLECTION)
    }

    pub async fn save_profile(&self, chat_id: i64, profile: UserProfile) -> Result<()> {
        let users = self.users_collection();
        let now = Utc::now();

        log::info!("🔄 Attempting to save profile for user {}", chat_id);

        // Get existing document if any
        let existing = users.find_one(mongodb::bson::doc! { "chat_id": chat_id }, None).await
            .map_err(|e| {
                log::error!("❌ Failed to query existing profile for user {}: {}", chat_id, e);
                anyhow::anyhow!("MongoDB query error: {}", e)
            })?;

        let doc = match existing {
            Some(mut doc) => {
                log::info!("📝 Updating existing profile for user {}", chat_id);
                doc.profile = Some(profile);
                doc.updated_at = now;
                doc
            },
            None => {
                log::info!("📝 Creating new profile document for user {}", chat_id);
                UserDocument {
                    chat_id,
                    profile: Some(profile),
                    moods: Vec::new(),
                    created_at: now,
                    updated_at: now,
                }
            }
        };

        users.replace_one(
            mongodb::bson::doc! { "chat_id": chat_id },
            doc,
            mongodb::options::ReplaceOptions::builder().upsert(true).build(),
        ).await.map_err(|e| {
            log::error!("❌ Failed to save profile for user {}: {}", chat_id, e);
            anyhow::anyhow!("MongoDB save error: {}", e)
        })?;

        log::info!("✅ Successfully saved profile for user {}", chat_id);
        Ok(())
    }

    pub async fn get_profile(&self, chat_id: i64) -> Result<Option<UserProfile>> {
        let users = self.users_collection();
        
        log::info!("🔄 Attempting to retrieve profile for user {}", chat_id);
        
        let result = users.find_one(mongodb::bson::doc! { "chat_id": chat_id }, None).await
            .map_err(|e| {
                log::error!("❌ Failed to retrieve profile for user {}: {}", chat_id, e);
                anyhow::anyhow!("MongoDB query error: {}", e)
            })?;
            
        match result {
            Some(user) => {
                log::info!("✅ Successfully retrieved profile for user {}", chat_id);
                Ok(user.profile)
            },
            None => {
                log::info!("ℹ️ No profile found for user {}", chat_id);
                Ok(None)
            }
        }
    }

    pub async fn save_mood(&self, chat_id: i64, mood: MoodEntry) -> Result<()> {
        let users = self.users_collection();
        let now = Utc::now();

        users.update_one(
            mongodb::bson::doc! { "chat_id": chat_id },
            mongodb::bson::doc! {
                "$push": { "moods": mongodb::bson::to_document(&mood)? },
                "$set": { "updated_at": now },
                "$setOnInsert": {
                    "chat_id": chat_id,
                    "created_at": now
                }
            },
            mongodb::options::UpdateOptions::builder().upsert(true).build(),
        ).await?;

        log::info!("✅ Saved mood entry for user {}", chat_id);
        Ok(())
    }

    pub async fn get_moods(&self, chat_id: i64) -> Result<Vec<MoodEntry>> {
        let users = self.users_collection();
        
        if let Some(user) = users.find_one(mongodb::bson::doc! { "chat_id": chat_id }, None).await? {
            log::info!("📖 Retrieved {} mood entries for user {}", user.moods.len(), chat_id);
            Ok(user.moods)
        } else {
            log::info!("❌ No mood entries found for user {}", chat_id);
            Ok(Vec::new())
        }
    }

    pub async fn get_mood_stats(&self) -> Result<std::collections::HashMap<String, i32>> {
        let users = self.users_collection();
        let mut stats = std::collections::HashMap::new();
        
        let mut cursor = users.find(None, None).await?;
        while let Some(user) = cursor.next().await {
            if let Ok(user) = user {
                for mood in user.moods {
                    *stats.entry(mood.mood).or_insert(0) += 1;
                }
            }
        }
        
        log::info!("📊 Retrieved mood statistics with {} different moods", stats.len());
        Ok(stats)
    }
} 