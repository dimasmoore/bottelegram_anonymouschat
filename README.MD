# 🤖 Telegram Anonymous Bot

<div align="center">

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Telegram](https://img.shields.io/badge/Telegram-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white)](https://telegram.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](LICENSE)

A secure and high-performance Telegram bot built in Rust that enables private and anonymous communication.

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Contributing](#contributing)

</div>

## ✨ Features

### Core Functionality
- 🔒 **Anonymous Chat System**
  - Random chat partner matching
  - Chat room creation and management
  - Private and group chat support
  - User profile customization

- 🎭 **Profile & Mood System**
  - Customizable user profiles with nicknames and emojis
  - Mood tracking and statistics
  - Anonymous mood sharing
  - Personal bio settings

- 🏰 **Chat Rooms**
  - Create custom chat rooms
  - Room size limits
  - Room listing and discovery
  - Easy join/leave functionality

### Technical Infrastructure
- 📊 **Data Management**
  - MongoDB integration for persistent storage
  - Redis for real-time data and caching
  - Efficient message queuing
  - Profile data management

- 🛡️ **Security & Filtering**
  - Content filtering system
  - Message sanitization
  - User privacy protection
  - Rate limiting controls

- ⚡ **Performance**
  - Asynchronous message processing
  - Optimized database queries
  - Efficient memory usage
  - Real-time message delivery

## 🚀 Prerequisites

- Rust (latest stable version)
- MongoDB
- Redis
- Telegram Bot Token (obtain from [@BotFather](https://t.me/BotFather))
- Cargo (Rust package manager)

## 📦 Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/telegram_anonymouse.git
cd telegram_bot
```

2. Create a `.env` file in the project root:
```env
TELEGRAM_BOT_TOKEN=your_bot_token_here
MONGODB_URI=your_mongodb_uri
REDIS_URL=your_redis_url
```

3. Build the project:
```bash
cargo build --release
```

## 🎮 Usage

1. Start the bot:
```bash
cargo run --release
```

2. Open Telegram and search for your bot

## 🤖 Bot Commands

| Command | Description | Usage |
|---------|-------------|-------|
| `/start` | 🎉 Start the bot | `/start` |
| `/help` | 📜 Show help message | `/help` |
| `/find` | 🔍 Find a random chat partner | `/find` |
| `/createroom` | 👋 Create a new chat room | `/createroom <name> <max_members>` |
| `/listrooms` | 📋 List available chat rooms | `/listrooms` |
| `/joinroom` | 🚪 Join a chat room | `/joinroom <room_id>` |
| `/leave` | 👋 Leave current chat or room | `/leave` |
| `/setprofile` | 👤 Set your profile | `/setprofile <nickname> <emoji> <bio>` |
| `/viewprofile` | 📝 View your profile | `/viewprofile` |
| `/setmood` | 😊 Set your mood | `/setmood <mood> <note>` |
| `/viewmood` | 📊 View your mood history | `/viewmood` |
| `/moodstats` | 📈 View anonymous mood statistics | `/moodstats` |
| `/broadcast` | 📢 Broadcast message | `/broadcast <message>` |

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔐 Security

Security is our top priority:
- All messages are encrypted in transit
- No personal data is stored
- Regular security audits
- Compliance with privacy regulations

## 💬 Support

Need help? We're here for you!

- 📫 Open an issue for bug reports
- 💡 Feature requests are welcome
- 📚 Check our [Wiki](../../wiki) for detailed documentation

---

<div align="center">
Made with ❤️ using Rust
</div>
