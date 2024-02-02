use crate::commands::{ClientCommand, Output};
use crate::common::{amount, private_key};
use crate::error::CliError;
use clap::{Arg, ArgMatches, Command};

pub struct Transfer;

fn recipient_arg() -> Arg {
    Arg::new("recipient")
        .long("recipient")
        .short('r')
        .required(true)
        .value_name("PUBLIC_KEY")
}

impl ClientCommand for Transfer {
    const NAME: &'static str = "transfer";
    const ABOUT: &'static str = "Transfers funds to another account";

    fn new() -> Command {
        Command::new(Self::NAME)
            .about(Self::ABOUT)
            .arg(recipient_arg())
            .arg(amount::arg())
            .arg(private_key::arg())
    }

    fn run(matches: &ArgMatches) -> Result<Output, CliError> {
        let amount = amount::get(matches)?;

        todo!();
    }
}
