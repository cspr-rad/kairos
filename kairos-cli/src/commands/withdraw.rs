use crate::client::{self, KairosClientError};
use crate::common::args::{AmountArg, NonceArg, PrivateKeyPathArg};
use crate::error::CliError;

use axum_extra::routing::TypedPath;
use casper_types::PublicKey;

use clap::Parser;
use kairos_server::routes::withdraw::WithdrawPath;
use kairos_server::routes::PayloadBody;
use kairos_tx::asn::{SigningPayload, Withdrawal};
use reqwest::Url;

#[derive(Parser)]
pub struct Args {
    #[clap(flatten)]
    amount: AmountArg,
    #[clap(flatten)]
    private_key_path: PrivateKeyPathArg,
    #[clap(flatten)]
    nonce: NonceArg,
}

pub fn run(args: Args, kairos_server_address: Url) -> Result<String, CliError> {
    let amount: u64 = args.amount.field;
    // TODO introduce public key path
    let signer_public_key = PublicKey::from_file(args.private_key_path.field).unwrap();

    let nonce = match args.nonce.val {
        None => client::get_nonce(&kairos_server_address, &signer_public_key)?,
        Some(nonce) => nonce,
    };

    // TODO: Create transaction and sign it with `signer`.

    // TODO: Send transaction to the network, using Rust SDK.
    let res = reqwest::blocking::Client::new()
        .post(kairos_server_address.join(WithdrawPath::PATH).unwrap())
        .json(&PayloadBody {
            public_key: signer_public_key.into(),
            payload: SigningPayload::new(nonce, Withdrawal::new(amount))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .send()
        .map_err(KairosClientError::from)?;

    if res.status().is_success() {
        Ok("Withdrawal successfully sent to L2".to_string())
    } else {
        Err(KairosClientError::ResponseErrorWithCode(
            res.status().as_u16(),
            res.text().unwrap_or_default(),
        )
        .into())
    }
}
