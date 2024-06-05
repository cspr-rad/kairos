use crate::client;
use crate::common::args::{AmountArg, ContractHashArg, PrivateKeyPathArg};
use crate::error::CliError;

use casper_client_types::{crypto::SecretKey, ContractHash};
use clap::Parser;
use hex::FromHex;
use reqwest::Url;

use kairos_crypto::error::CryptoError;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    amount: AmountArg,
    #[clap(flatten)]
    private_key_path: PrivateKeyPathArg,
    #[clap(flatten)]
    contract_hash: ContractHashArg,
}

pub fn run(args: Args, kairos_server_address: Url) -> Result<String, CliError> {
    let contract_hash = args.contract_hash.field;
    let amount: u64 = args.amount.field;
    let path = args.private_key_path.field;
    let depositor_secret_key =
        SecretKey::from_file(&path).map_err(|err| CryptoError::FailedToParseKey {
            error: err.to_string(),
        })?;

    let contract_hash_bytes = <[u8; 32]>::from_hex(contract_hash)?;
    let contract_hash = ContractHash::new(contract_hash_bytes);

    client::deposit(
        &kairos_server_address,
        &depositor_secret_key,
        &contract_hash,
        amount,
    )
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
