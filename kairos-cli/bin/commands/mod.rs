pub mod deposit;
pub mod transfer;
pub mod withdraw;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Deposits funds into your account")]
    Deposit(deposit::Args),
    #[command(about = "Transfers funds to another account")]
    Transfer(transfer::Args),
    #[command(about = "Withdraws funds from your account")]
    Withdraw(withdraw::Args),
}
