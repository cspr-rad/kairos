pub mod client;
pub mod commands;
pub mod common;
pub mod error;
pub mod utils;

use crate::error::CliError;

use clap::{Parser, Subcommand};
use reqwest::Url;

#[derive(Parser)]
#[command(name = "Kairos Client", about = "CLI for interacting with Kairos")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(long, value_name = "URL", default_value = "http://0.0.0.0:9999")]
    pub kairos_server_address: Url,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Deposits funds into your account")]
    Deposit(commands::deposit::Args),
    #[command(about = "Transfers funds to another account")]
    Transfer(commands::transfer::Args),
    #[command(about = "Withdraws funds from your account")]
    Withdraw(commands::withdraw::Args),
}

pub fn run(
    Cli {
        command,
        kairos_server_address,
    }: Cli,
) -> Result<String, CliError> {
    match command {
        Command::Deposit(args) => commands::deposit::run(args, kairos_server_address),
        Command::Transfer(args) => commands::transfer::run(args),
        Command::Withdraw(args) => commands::withdraw::run(args),
    }
}
