use std::fs;
use std::path::Path;

fn main() {
    let src = Path::new("rules.lua");
    let dst = Path::new("target/debug/rules.lua");
    fs::copy(src, dst).expect("Failed to copy rules.lua");
}