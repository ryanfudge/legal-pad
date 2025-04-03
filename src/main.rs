mod utils;

use clap::Parser;
use utils::file_writing::write_to_file;
use crate::utils::argparse::Cli;

fn main() {
    let args = Cli::parse();

    println!("Category: {:?}", args.category);
    println!("Text: {}", args.text);

    write_to_file(args.category.as_deref(), &args.text).expect("Failed to write to file");
}
