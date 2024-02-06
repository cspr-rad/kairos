use crate::commands::{ClientCommand, Output};
use crate::common::{amount, private_key};
use crate::crypto::signer::CasperSigner;
use crate::error::CliError;
use clap::{ArgMatches, Command};

pub struct Deposit;

impl ClientCommand for Deposit {
    const NAME: &'static str = "deposit";
    const ABOUT: &'static str = "Deposits funds into your account";

    fn new_cmd() -> Command {
        Command::new(Self::NAME)
            .about(Self::ABOUT)
            .arg(amount::arg())
            .arg(private_key::arg())
    }

    fn run(matches: &ArgMatches) -> Result<Output, CliError> {
        let _amount = amount::get(matches)?;
        let private_key = private_key::get(matches)?;

        let _signer = CasperSigner::from_key(private_key);

        // TODO: Create transaction and sign it with `signer`.

        // TODO: Send transaction to the network, using Rust SDK.

        Ok("ok".to_string())
    }
}
