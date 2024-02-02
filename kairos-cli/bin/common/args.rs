use crate::error::CliError;
use clap::{Arg, ArgMatches};

pub mod amount {
    use super::*;

    const ARG_NAME: &str = "amount";
    const ARG_SHORT: char = 'a';
    const ARG_VALUE_NAME: &str = "NUM_MOTES";

    pub fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .short(ARG_SHORT)
            .required(true)
            .value_name(ARG_VALUE_NAME)
    }

    pub fn get(matches: &ArgMatches) -> Result<u64, CliError> {
        let value = matches
            .get_one::<String>(ARG_NAME)
            .map(String::as_str)
            .ok_or_else(|| CliError::MissingArgument { context: ARG_NAME })?;

        let amount = value
            .parse::<u64>()
            .map_err(|_| CliError::FailedToParseU64 { context: "amount" })?;

        Ok(amount)
    }
}

pub mod private_key {
    use super::*;

    pub fn arg() -> Arg {
        Arg::new("private-key")
            .long("private-key")
            .short('k')
            .required(true)
            .value_name("FILE_PATH")
    }
}
