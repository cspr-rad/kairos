mod deposit;
mod transfer;
mod withdraw;

pub use deposit::Deposit;
pub use transfer::Transfer;
pub use withdraw::Withdraw;

use clap::{ArgMatches, Command};

pub trait ClientCommand {
    const NAME: &'static str;
    const ABOUT: &'static str;

    /// Constructs the clap subcommand.
    fn new() -> Command;

    /// Parses the arg matches and runs the subcommand.
    /// TODO: Return execution result.
    fn run(matches: &ArgMatches);
}
