use crate::commands::{ClientCommand, Output};
use crate::common::{amount, private_key};
use crate::error::CliError;
use clap::{ArgMatches, Command};

pub struct Deposit;

impl ClientCommand for Deposit {
    const NAME: &'static str = "deposit";
    const ABOUT: &'static str = "Deposits funds into your account";

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
