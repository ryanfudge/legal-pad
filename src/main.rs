mod utils;

use clap::Parser;
use utils::file_writing::write_to_file;
use crate::utils::argparse::Cli;

fn main() {
    let args = Cli::parse();

    println!("Category: {:?}", args.category);
    println!("Text: {}", args.text);

    const FILE_NAME : &str = "test.txt";
    write_to_file(FILE_NAME, &args.text).expect("Failed to write to file");
}
