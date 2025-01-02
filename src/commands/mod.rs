use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "✨ Available commands:")]
pub enum Command {
    #[command(description = "📜 Show this help message")]
    Help,
    #[command(description = "🎉 Start the bot")]
    Start,
    #[command(description = "🔍 Find a random chat partner")]
    Find,
    #[command(description = "👋 Create a new chat room (usage: /createroom <name> <max_members>)", parse_with = "split")]
    CreateRoom {
        name: String,
        max_members: String,
    },
    #[command(description = "📋 List available chat rooms")]
    ListRooms,
    #[command(description = "🚪 Join a chat room (usage: /joinroom <room_id>)")]
    JoinRoom(String),
    #[command(description = "👋 Leave current chat or room")]
    Leave,
    #[command(description = "👤 Set your profile (usage: /setprofile <nickname> <emoji> <bio>)", parse_with = "split")]
    SetProfile {
        nickname: String,
        emoji: String,
        bio: String,
    },
    #[command(description = "📝 View your profile")]
    ViewProfile,
    #[command(description = "😊 Set your mood (usage: /setmood <mood> <note>)", parse_with = "split")]
    SetMood {
        mood: String,
        note: String,
    },
    #[command(description = "📊 View your mood history")]
    ViewMood,
    #[command(description = "📈 View anonymous mood statistics")]
    MoodStats,
    #[command(description = "📢 Broadcast message (usage: /broadcast <message>)")]
    Broadcast(String),
} 