mod utils;

use clap::{Parser, Subcommand};
use utils::file_writing::write_to_file;
use utils::viewer::view_notes;

#[derive(Parser)]
#[command(name = "pad", about = "A notepad for quick thoughts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new note
    Add {
        /// Category for the note
        #[arg(short, long)]
        category: Option<String>,

        /// The text content to be saved
        text: String,
    },
    /// View all notes
    View,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { category, text } => {
            println!("Category: {:?}", category);
            println!("Text: {}", text);
            write_to_file(category.as_deref(), &text).expect("Failed to write to file");
        }
        Commands::View => {
            view_notes().expect("Failed to view notes");
        }
    }
}
