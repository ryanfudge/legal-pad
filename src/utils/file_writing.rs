/*
I will write to the same file each time, similar to how command history works but not really lol
*/

use std::fs;
use std::io::Write;
use chrono::Local;

const NOTES_FILE: &str = "notes.txt";

pub fn write_to_file(category: Option<&str>, content: &str) -> std::io::Result<()> {
    // Open the file and append new content to it
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(NOTES_FILE)?;

    // Write content with timestamp and category
    let timestamp = Local::now().format("%Y-%m-%d");
    let category = category.unwrap_or("general");
    writeln!(
        file, 
        "{:<14} {:<10} {}",
        format!("[{}]", timestamp), 
        format!("[{}]", category), 
        content
    )?;
    Ok(())
}