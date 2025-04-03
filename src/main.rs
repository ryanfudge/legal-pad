use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "pad", about = "A notepad for quick thoughts")]
struct Cli {
    #[arg(short, long)]
    category: Option<String>, // category

    text: String // text that is being saved
}

fn main() {
    let args = Cli::parse();
}
