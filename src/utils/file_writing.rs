/*
I will write to the same file each time, similar to how command history works but not really lol
*/

use std::fs;
use std::io::Write;

pub fn write_to_file(file_name: &str, content: &str) -> std::io::Result<()> {
    // Open the file and append new content to it
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(file_name)?;

    // Write content
    file.write_all(content.as_bytes())?;
    file.write_all(b"\n")?; // Add a newline after each entry
    Ok(())
}