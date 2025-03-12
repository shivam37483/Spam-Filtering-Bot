# Spam Bot MVP

## Overview

`spam-bot-mvp` is a Telegram spam detection bot built as a Minimum Viable Product (MVP). The bot monitors messages in Telegram group chats, detects spam using custom Lua-based rules, and notifies administrators when spam is detected. It also provides commands for users to report spam and for admins to add custom rules, making it a flexible and extensible spam filtering solution.

The bot is written in Rust using the `teloxide` framework for Telegram API interactions, `rusqlite` for persistent storage, and `rlua` for evaluating custom spam rules. It features a modular design with separate components for rule management, utility functions, and bot logic, ensuring maintainability and scalability.


## Features

### Current Functionalities
- **Spam Detection**:
  - Automatically checks incoming text messages in group chats for spam.
  - Uses custom rules defined in a `rules.lua` script to assign scores to messages.
  - Flags a message as spam if its score is ≥ 5.0 (e.g., "spam" scores 10.0, "http" scores 5.0).
  - Increments the sender's spam score when a message is flagged as spam.

- **Admin Notifications**:
  - Notifies admins when spam is detected with details (message text, sender ID, spam score).
  - Attempts to send private messages to group admins; falls back to group notifications if private messaging fails or no admins are found.

- **Bot Commands**:
  - `/start`: Displays a welcome message ("Hello! I'm a spam filter bot.").
  - `/report`: Allows users to report a message as spam by replying to it. The bot evaluates the message and confirms if it’s spam.
  - `/add_rule <keyword> <score>`: Allows admins to add custom spam rules (e.g., `/add_rule spam 10.0`).

- **Admin Privileges**:
  - Only group admins can use `/add_rule` to modify spam detection rules.
  - The bot checks admin status using the Telegram API.

- **Persistent Storage**:
  - Stores rules and sender scores in a SQLite database (`rules.db`).
  - Tracks each sender’s spam score and message count for future enhancements (e.g., auto-muting).

- **Logging**:
  - Logs bot activity, spam detection events, and errors using the `log` crate and `env_logger`.
  - Provides detailed logs for debugging (e.g., message scores, notification attempts).

### File Structure
- **`Cargo.toml`**: Defines project dependencies, including `teloxide`, `rusqlite`, `rlua`, `log`, `dotenv`, and test dependencies (`tempfile`, `mockall`, `tokio-test`).
- **`main.rs`**: The entry point of the bot, handling Telegram events, commands, and message checks.
- **`rules.rs`**: Manages spam detection rules and sender scores using SQLite and Lua.
- **`utils.rs`**: Contains utility functions for checking admin status and notifying admins.
- **`build.rs`**: A build script that copies `rules.lua` to the `target/debug` directory during builds.
- **`rules.lua`**: Defines custom spam detection rules in Lua (e.g., scoring "spam" as 10.0, "http" as 5.0).


## Setup Instructions

### Prerequisites
- **Rust**: Ensure Rust is installed (version 1.60+ recommended). Install via [rustup](https://rustup.rs/).
- **Telegram Bot Token**: Obtain a bot token from `@BotFather` on Telegram.
- **SQLite**: No installation required (`rusqlite` uses a bundled SQLite).

### Installation
1. **Clone the Repository**:
    ```bash
    git clone https://github.com/shivam37483/Spam-Filtering-Bot.git
    cd spam-bot-mvp
    ```

2. **Set Up Environment Variables**:
    - Create a **.env** file in the project root:
    ```plaintext
    TELOXIDE_TOKEN=your_bot_token_here
    ```

    - Replace your_bot_token_here with the token from @BotFather.

3. **Build the Project**:
    ```bash
    cargo build
    ```

    - The **build.rs** script will copy **rules.lua** to **target/debug/rules.lua**.

4. **Control Logging**
    - Windows
    ```sh
    set RUST_LOG=info
    ```

    - Mac/Linux
    ```sh
    export RUST_LOG=info
    ```

    Log levels include error, warn, info, debug, and trace.
   
6. **Run the Bot**:
    ```bash
    cargo run
    ```

    - The bot will start and print "Bot started!" to the console.

7. Add the Bot to a Telegram Group
   - Add the bot to a Telegram group via its username (e.g., **@spam_detection_rapamd_bot**).
   - Make the bot an admin in the group to fetch the admin list (required for private notifications).
   - Send **/start** in the group to initialize the bot.


## Usage

   - Test Spam Detection:
       - Send a message like "spam" or "http" in the group.
       - The bot will respond with "Spam detected! Admins notified." and provide details (e.g., "Spam detected: spam\nSender ID: 5009223754\nSpam Score: 1").
   - Report a Message:
       - Reply to a message with /report to check if it’s spam.
     
   - Add a Custom Rule (Admins Only):
       - Use /add_rule spam 10.0 to add a new rule.

   - Check Logs:
       - View the terminal for logs (e.g., message scores, notification attempts).

## Documentation

  - Rustdoc: Generate and view the documentation:
    ```sh
      cargo doc --no-deps --open
    ```

  - Remove <--no-deps> for complete documentation including that for all the dependencies:
    ```sh
      cargo doc --open
    ```

    - Access the HTML output at target/doc/spam-filtering-bot/index.html.

## Testing

The project includes **Unit Tests** to verify individual functions:

Run tests with:

```sh
cargo test
```

All tests pass, ensuring the application’s core functionality is robust.


# Future Prospects

## 1. Enhanced Spam Detection

### Advanced Rule System:
- Support regular expressions in `rules.lua` for more complex pattern matching (e.g., URLs, email addresses).
- Allow rules to consider message context (e.g., sender history, message frequency).

### Machine Learning Integration:
- Integrate a machine learning model (e.g., using `rust-bert`) to classify messages as spam based on training data.
- Use sender behavior patterns (e.g., rapid messaging) to improve detection accuracy.

## 2. Admin Features

### Private Notifications:
- Fix private notifications to admins by addressing Telegram API restrictions (e.g., ensuring `/start` in private chats, handling rate limits).
- Add retry logic for failed notifications.

### Admin Commands:
- Add `/remove_rule <keyword>` to delete rules.
- Add `/list_rules` to display all active rules.
- Add `/mute <user_id>` to allow admins to mute spammers directly.

## 3. User Features

### Spam Reporting Feedback:
- Provide more detailed feedback for `/report` (e.g., breakdown of the score).
- Allow users to appeal false positives via a command (e.g., `/appeal`).

### User Statistics:
- Add a command (e.g., `/stats`) to show a user’s spam score and message count.

## 4. Automation

### Auto-Muting/Banning:
- Automatically mute or ban users who exceed a spam score threshold (e.g., `10`).
- Notify admins before taking action, with an option to override.

### Rate Limiting:
- Detect and flag rapid message sending as potential spam behavior.

## 5. Persistence and Scalability

### Database Improvements:
- Add a uniqueness constraint to the rules table to prevent duplicate keywords.
- Optimize database queries for large groups with many users.

### Configuration File:
- Move settings (e.g., spam score threshold, database path) to a configuration file (e.g., `config.toml`).

## 6. Deployment and Monitoring

### Docker Support:
- Create a `Dockerfile` for easy deployment on servers.
- Include a `docker-compose.yml` for running the bot with a SQLite database.

### Monitoring and Alerts:
- Integrate a monitoring system (e.g., Prometheus) to track bot performance (e.g., message processing rate, error rate).
- Send alerts to maintainers if the bot crashes or encounters critical errors.

## 7. Testing and Reliability

### Expanded Test Suite:
- Add more unit tests for edge cases (e.g., concurrent message handling, large rule sets).
- Add integration tests to simulate Telegram API interactions.

### Error Handling:
- Improve error handling for Telegram API failures (e.g., rate limits, network issues).
- Add graceful shutdown on `Ctrl+C` to save the database state.

## 8. Cross-Platform Support

### Release Builds:
- Update `build.rs` to support release builds by copying `rules.lua` to `target/release`.
- Use `OUT_DIR` for dynamic output paths.

### Multi-Platform Compatibility:
- Ensure the bot works on Windows, Linux, and macOS by testing file paths and SQLite behavior.
