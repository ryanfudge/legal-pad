mod utils;

use clap::Parser;
use crate::utils::argparse::Cli;

fn main() {
    let args = Cli::parse();
    println!("Category: {:?}", args.category);
    println!("Text: {}", args.text);
}
