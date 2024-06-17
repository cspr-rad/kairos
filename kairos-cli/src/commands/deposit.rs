use crate::client;
use crate::common::args::{AmountArg, PrivateKeyPathArg};
use crate::error::CliError;

use casper_types::crypto::SecretKey;
use clap::Parser;
use reqwest::Url;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    amount: AmountArg,
    #[clap(flatten)]
    private_key_path: PrivateKeyPathArg,
}

pub fn run(args: Args, kairos_server_address: Url) -> Result<String, CliError> {
    let amount: u64 = args.amount.field;
    let path = args.private_key_path.field;
    let depositor_secret_key = SecretKey::from_file(&path)
        .map_err(|err| panic!("Failed to read secret key from file {:?}: {}", path, err))
        .unwrap();

    client::deposit(&kairos_server_address, &depositor_secret_key, amount)
        .map_err(Into::<CliError>::into)
        .map(|deploy_hash| {
            // to_string crops the hash to <hash-prefix>..<hash-postfix>
            // thus we use serde to get the full string, and remove the
            // double quotes that get added during serialization
            let mut output: String = serde_json::to_string(&deploy_hash)
                .unwrap()
                .chars()
                .filter(|&c| c != '"') // Filter out the double quotes
                .collect();
            output.push('\n');
            output
        })
}
