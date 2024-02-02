use crate::commands::{ClientCommand, Output};
use crate::common::{amount, private_key};
use crate::error::CliError;
use clap::{ArgMatches, Command};

pub struct Withdraw;

impl ClientCommand for Withdraw {
    const NAME: &'static str = "withdraw";
    const ABOUT: &'static str = "Withdraws funds from your account";

    fn new() -> Command {
        Command::new(Self::NAME)
            .about(Self::ABOUT)
            .arg(amount::arg())
            .arg(private_key::arg())
    }

    fn run(matches: &ArgMatches) -> Result<Output, CliError> {
        todo!();
    }
}
