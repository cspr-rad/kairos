mod commands;
mod common;
mod crypto;
mod error;
mod utils;

use std::process;

use clap::Parser;
use commands::Command;

#[derive(Parser)]
#[command(name = "Kairos Client", about = "CLI for interacting with Kairos")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Deposit(args) => commands::deposit::run(args),
        Command::Transfer(args) => commands::transfer::run(args),
        Command::Withdraw(args) => commands::withdraw::run(args),
    };

    match result {
        Ok(output) => {
            println!("{}", output)
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    }
}
