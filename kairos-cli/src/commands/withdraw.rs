use crate::client::KairosClientError;
use crate::common::args::{AmountArg, NonceArg, PrivateKeyPathArg};
use crate::error::CliError;

use kairos_crypto::error::CryptoError;
use kairos_crypto::implementations::Signer;
use kairos_crypto::{SignerCore, SignerFsExtension};

use clap::Parser;
use kairos_server::routes::transfer::TransferPath;
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

pub async fn run(args: Args, kairos_server_address: Url) -> Result<String, CliError> {
    let amount: u64 = args.amount.field;
    let signer =
        Signer::from_private_key_file(args.private_key_path.field).map_err(CryptoError::from)?;
    let nonce = args.nonce.val;

    // TODO: Create transaction and sign it with `signer`.

    // TODO: Send transaction to the network, using Rust SDK.
    reqwest::Client::new()
        .post(kairos_server_address.join(WithdrawPath::PATH).unwrap())
        .json(&PayloadBody {
            public_key: signer.to_public_key()?,
            payload: SigningPayload::new(nonce, Withdrawal::new(amount))
                .try_into()
                .unwrap(),
            signature: vec![],
        })
        .send()
        .await
        .map_err(KairosClientError::from)?;

    Ok("ok".to_string())
}
