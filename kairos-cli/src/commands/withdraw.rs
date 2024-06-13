use crate::common::args::{AmountArg, PrivateKeyPathArg};
use crate::error::CliError;

use kairos_crypto::error::CryptoError;
use kairos_crypto::implementations::Signer;
use kairos_crypto::SignerCore;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[clap(flatten)]
    amount: AmountArg,
    #[clap(flatten)]
    private_key_path: PrivateKeyPathArg,
}

pub fn run(args: Args) -> Result<String, CliError> {
    let _amount: u64 = args.amount.field;
    let _signer =
        Signer::from_private_key_file(args.private_key_path.field).map_err(CryptoError::from)?;

    // TODO: Create transaction and sign it with `signer`.

    // TODO: Send transaction to the network, using Rust SDK.

    Ok("ok".to_string())
}
