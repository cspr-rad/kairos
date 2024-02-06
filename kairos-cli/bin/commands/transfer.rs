use crate::common::{amount, private_key};
use crate::crypto::public_key::CasperPublicKey;
use crate::crypto::signer::CasperSigner;
use crate::error::CliError;
use clap::{Arg, ArgMatches, Command};

pub struct Transfer;

const ARG_NAME: &str = "recipient";
const ARG_SHORT: char = 'r';
const ARG_VALUE_NAME: &str = "PUBLIC_KEY";

pub mod recipient {
    use super::*;

    pub fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .short(ARG_SHORT)
            .required(true)
            .value_name(ARG_VALUE_NAME)
    }

    pub fn get(matches: &ArgMatches) -> Result<CasperPublicKey, CliError> {
        let value = matches
            .get_one::<String>("recipient")
            .map(String::as_str)
            .unwrap();

        CasperPublicKey::from_hex(value).map_err(|error| CliError::CryptoError { error })
    }
}

impl Transfer {
    pub const NAME: &'static str = "transfer";
    pub const ABOUT: &'static str = "Transfers funds to another account";

    pub fn new_cmd() -> Command {
        Command::new(Self::NAME)
            .about(Self::ABOUT)
            .arg(recipient::arg())
            .arg(amount::arg())
            .arg(private_key::arg())
    }

    pub fn run(matches: &ArgMatches) -> Result<String, CliError> {
        let _recipient = recipient::get(matches)?;
        let _amount = amount::get(matches)?;
        let private_key = private_key::get(matches)?;

        let _signer = CasperSigner::from_key(private_key);

        // TODO: Create transaction and sign it with `signer`.

        // TODO: Send transaction to the network, using Rust SDK.

        Ok("ok".to_string())
    }
}
