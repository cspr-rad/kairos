mod deposit;
mod transfer;
mod withdraw;

pub use deposit::Deposit;
pub use transfer::Transfer;
pub use withdraw::Withdraw;

use crate::error::CliError;
use clap::{ArgMatches, Command};

// NOTE: Temporarily we use plain output.
pub type Output = String;

pub trait ClientCommand {
    const NAME: &'static str;
    const ABOUT: &'static str;

    /// Constructs the clap subcommand.
    fn new() -> Command;

    /// Parses the arg matches and runs the subcommand.
    fn run(matches: &ArgMatches) -> Result<Output, CliError>;
}
