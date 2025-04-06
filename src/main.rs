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
        /// Mark as something to read
        #[arg(short = 'r', long = "read")]
        read: bool,

        /// Mark as something to watch
        #[arg(short = 'w', long = "watch")]
        watch: bool,

        /// Mark as something to listen to
        #[arg(short = 'l', long = "listen")]
        listen: bool,

        /// Mark as an idea
        #[arg(short = 'i', long = "idea")]
        idea: bool,

        /// The text content to be saved
        text: String,
    },
    /// View all notes
    View,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { read, watch, listen, idea, text } => {
            let category = if read {
                "read"
            } else if watch {
                "watch"
            } else if listen {
                "listen"
            } else if idea {
                "idea"
            } else {
                "general"
            };
            write_to_file(Some(category), &text).expect("Failed to write to file");
        }
        Commands::View => {
            view_notes().expect("Failed to view notes");
        }
    }
}
