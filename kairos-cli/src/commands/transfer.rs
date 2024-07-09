use crate::client::{self, KairosClientError};
use crate::common::args::{AmountArg, NonceArg, PrivateKeyPathArg, RecipientArg};
use crate::error::CliError;

use axum_extra::routing::TypedPath;
use kairos_crypto::error::CryptoError;
use kairos_crypto::implementations::Signer;
use kairos_crypto::SignerCore;
use kairos_crypto::SignerFsExtension;

use clap::Parser;
use kairos_server::routes::{transfer::TransferPath, PayloadBody};
use kairos_tx::asn::{SigningPayload, Transfer};
use reqwest::Url;

#[derive(Parser)]
pub struct Args {
    #[clap(flatten)]
    recipient: RecipientArg,
    #[clap(flatten)]
    amount: AmountArg,
    #[clap(flatten)]
    private_key_path: PrivateKeyPathArg,
    #[clap(flatten)]
    nonce: NonceArg,
}

pub fn run(args: Args, kairos_server_address: Url) -> Result<String, CliError> {
    let recipient = Signer::from_public_key(args.recipient.recipient)?.to_public_key()?;
    let amount: u64 = args.amount.field;
    let signer =
        Signer::from_private_key_file(args.private_key_path.field).map_err(CryptoError::from)?;
    let signer_public_key = signer.to_public_key()?;
    let nonce = match args.nonce.val {
        None => client::get_nonce(&kairos_server_address, &signer_public_key)?,
        Some(nonce) => nonce,
    };

    // TODO: Create transaction and sign it with `signer`.

    // TODO: Send transaction to the network, using Rust SDK.
    reqwest::blocking::Client::new()
        .post(kairos_server_address.join(TransferPath::PATH).unwrap())
        .json(&PayloadBody {
            public_key: signer_public_key,
            payload: SigningPayload::new(nonce, Transfer::new(recipient, amount))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .send()
        .map_err(KairosClientError::from)?;

    Ok("ok".to_string())
}
