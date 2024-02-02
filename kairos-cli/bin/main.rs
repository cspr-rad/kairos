mod commands;
mod common;

use clap::Command;
use commands::{ClientCommand, Deposit, Transfer, Withdraw};

fn main() {
    let cli = Command::new("Kairos Client")
        .about("CLI for interacting with Kairos")
        .subcommand(Deposit::new())
        .subcommand(Transfer::new())
        .subcommand(Withdraw::new());
    cli.get_matches();
}
