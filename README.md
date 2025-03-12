# Spam Bot MVP

## Overview

`spam-bot-mvp` is a Telegram spam detection bot built as a Minimum Viable Product (MVP) for a Google Summer of Code (GSoC) midterm project. The bot monitors messages in Telegram group chats, detects spam using custom Lua-based rules, and notifies administrators when spam is detected. It also provides commands for users to report spam and for admins to add custom rules, making it a flexible and extensible spam filtering solution.

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

5. **Run the Bot**:
    ```bash
    