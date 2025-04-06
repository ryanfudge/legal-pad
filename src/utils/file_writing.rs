use std::fs;
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use dirs::home_dir;

const NOTES_FILE: &str = "notes.txt";

fn get_notes_path() -> PathBuf {
    let mut path = home_dir().expect("Could not find home directory");
    path.push("notes");
    fs::create_dir_all(&path).expect("Failed to create notes directory");
    path.push(NOTES_FILE);
    path
}

pub fn write_to_file(category: Option<&str>, content: &str) -> std::io::Result<()> {
    let notes_path = get_notes_path();
    
    // Open the file and append new content to it
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(notes_path)?;

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