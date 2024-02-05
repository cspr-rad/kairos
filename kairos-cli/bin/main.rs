mod commands;
mod common;
mod crypto;
mod error;

use clap::Command;
use commands::{ClientCommand, Deposit, Transfer, Withdraw};
use std::process;

fn cli() -> Command {
    Command::new("Kairos Client")
        .about("CLI for interacting with Kairos")
        .subcommand(Deposit::new())
        .subcommand(Transfer::new())
        .subcommand(Withdraw::new())
}

fn main() {
    let arg_matches = cli().get_matches();
    let (subcommand_name, matches) = arg_matches.subcommand().unwrap_or_else(|| {
        // No subcommand provided by user.
        let _ = cli().print_long_help();
        process::exit(1);
    });

    let result = match subcommand_name {
        Deposit::NAME => Deposit::run(matches),
        Transfer::NAME => Transfer::run(matches),
        Withdraw::NAME => Withdraw::run(matches),
        _ => {
            // This should not happen, unless we missed some subcommand.
            let _ = cli().print_long_help();
            process::exit(1);
        }
    };

    match result {
        Ok(output) => {
            println!("{}", output)
        }
        Err(error) => {
            println!("{}", error);
            process::exit(1);
        }
    }
}
