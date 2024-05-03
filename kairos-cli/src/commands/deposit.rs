use crate::client;
use crate::common::args::{AmountArg, PrivateKeyPathArg};
use crate::error::CliError;

use reqwest::Url;

use kairos_crypto::error::CryptoError;
use kairos_crypto::implementations::Signer;
use kairos_crypto::CryptoSigner;
use kairos_server::routes::PayloadBody;
use kairos_tx::asn::SigningPayload;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    amount: AmountArg,
    #[clap(flatten)]
    private_key_path: PrivateKeyPathArg,
}

pub fn run(args: Args, kairos_server_address: Url) -> Result<String, CliError> {
    let amount: u64 = args.amount.field;
    let signer =
        Signer::from_private_key_file(args.private_key_path.field).map_err(CryptoError::from)?;

    let client = reqwest::Client::new();
    let public_key = signer.to_public_key()?;

    let payload = SigningPayload::new_deposit(amount)
        .try_into()
        .expect("Failed serialize the deposit payload to bytes");
    let signature = signer.sign(&payload)?;
    let deposit_request = PayloadBody {
        public_key,
        payload,
        signature,
    };

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(client::submit_transaction_request(
            &client,
            &kairos_server_address,
            &deposit_request,
        ))
        .map_err(Into::<CliError>::into)
        .map(|_| "ok".to_string())
}
