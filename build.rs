use std::fs;
use std::path::Path;
/// Build script for copying the `rules.lua` script to the target directory.
///
/// This script ensures that the `rules.lua` file, which contains custom spam detection rules
/// written in Lua, is copied from the project root to the `target/debug` directory during the
/// build process. This makes the script accessible to the `spam-bot-mvp` application at runtime.
///
/// The script uses the `std::fs` module for file operations and `std::path::Path` for handling
/// file paths. If the copy operation fails, the script will panic with an error message.
///
/// # Panics
/// * Panics if the `rules.lua` file cannot be copied to `target/debug/rules.lua`. This could
///   happen if:
///   - The source file (`rules.lua`) does not exist in the project root.
///   - The `target/debug` directory cannot be written to due to permissions issues.
///   - Other I/O errors occur during the copy operation.
///
/// # Notes
/// * This script assumes that `rules.lua` exists in the project root directory.
/// * The destination path is hard-coded to `target/debug/rules.lua`, which means the script
///   only works correctly in debug builds. For release builds, the path should be adjusted
///   to `target/release/rules.lua` to match the build profile.
///
/// # Example
/// To use this build script, ensure a `rules.lua` file exists in the project root:
/// ```lua
/// -- rules.lua
/// function check_spam(message)
///     if string.lower(message):find("spam") then
///         return 10
///     end
///     return 0
/// end
/// ```
/// Running `cargo build` will copy this file to `target/debug/rules.lua`.
fn main() {
    let src = Path::new("rules.lua");
    let dst = Path::new("target/debug/rules.lua");
    fs::copy(src, dst).expect("Failed to copy rules.lua");
}