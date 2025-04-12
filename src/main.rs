mod utils;

use clap::{Parser, Subcommand};
use utils::file_writing::write_to_file;
use utils::viewer::view_notes;
use utils::semantic_search::SemanticSearch;

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
    /// Search notes semantically
    Search {
        /// The search query
        query: String,
        
        /// Number of results to return
        #[arg(short = 'k', long = "k-results", default_value = "5")]
        k: usize,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            write_to_file(Some(category), &text)?;
            
            // Add to semantic search index
            let mut semantic_search = SemanticSearch::new()?;
            semantic_search.add_note(&text)?;
        }
        Commands::View => {
            view_notes()?;
        }
        Commands::Search { query, k } => {
            let semantic_search = SemanticSearch::new()?;
            let results = semantic_search.search(&query, k)?;
            
            println!("\nSemantic search results for: '{}'", query);
            println!("----------------------------------------");
            for (i, (text, distance)) in results.iter().enumerate() {
                println!("{}. {} (distance: {:.4})", i + 1, text, distance);
            }
        }
    }
    Ok(())
}
