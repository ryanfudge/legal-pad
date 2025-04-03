use clap::Parser;

/// Command line interface for the legal-pad application
#[derive(Parser, Debug)]
#[command(name = "pad", about = "A notepad for quick thoughts")]
pub struct Cli {
    /// Category to organize the note under
    #[arg(short, long)]
    pub category: Option<String>,

    /// The text content to be saved
    pub text: String,
}