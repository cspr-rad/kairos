use clap::Parser;
use std::process;

fn main() {
    let args = kairos_cli::Cli::parse();
    match kairos_cli::run(args) {
        Ok(output) => {
            println!("{}", output)
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    }
}
