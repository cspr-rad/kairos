use crate::client::KairosClientError;
use crate::common::args::{AmountArg, NonceArg, PrivateKeyPathArg};
use crate::error::CliError;
use crate::utils::parse_hex_string;

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
    #[arg(long, short, value_name = "PUBLIC_KEY", value_parser = parse_hex_string)]
    recipient: ::std::vec::Vec<u8>, // Absolute path is required here - see https://github.com/clap-rs/clap/issues/4626#issue-1528622454.
    #[clap(flatten)]
    amount: AmountArg,
    #[clap(flatten)]
    private_key_path: PrivateKeyPathArg,
    #[clap(flatten)]
    nonce: NonceArg,
}

pub fn run(args: Args, kairos_server_address: Url) -> Result<String, CliError> {
    let recipient = Signer::from_public_key(args.recipient)?.to_public_key()?;
    let amount: u64 = args.amount.field;
    let signer =
        Signer::from_private_key_file(args.private_key_path.field).map_err(CryptoError::from)?;
    let nonce = args.nonce.val;

    // TODO: Create transaction and sign it with `signer`.

    // TODO: Send transaction to the network, using Rust SDK.
    reqwest::blocking::Client::new()
        .post(kairos_server_address.join(TransferPath::PATH).unwrap())
        .json(&PayloadBody {
            public_key: signer.to_public_key()?,
            payload: SigningPayload::new(nonce, Transfer::new(recipient, amount))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .send()
        .map_err(KairosClientError::from)?;

    Ok("ok".to_string())
}
