use crate::common::args::{AmountArg, PrivateKeyPathArg};
use crate::error::CliError;
use crate::utils::parse_hex_string;

use kairos_crypto::error::CryptoError;
use kairos_crypto::implementations::Signer;
use kairos_crypto::SignerCore;
use kairos_crypto::SignerFsExtension;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(long, short, value_name = "PUBLIC_KEY", value_parser = parse_hex_string)]
    recipient: ::std::vec::Vec<u8>, // Absolute path is required here - see https://github.com/clap-rs/clap/issues/4626#issue-1528622454.
    #[clap(flatten)]
    amount: AmountArg,
    #[clap(flatten)]
    private_key_path: PrivateKeyPathArg,
}

pub fn run(args: Args) -> Result<String, CliError> {
    let _recipient = Signer::from_public_key(args.recipient)?.to_public_key()?;
    let _amount: u64 = args.amount.field;
    let _signer =
        Signer::from_private_key_file(args.private_key_path.field).map_err(CryptoError::from)?;

    // TODO: Create transaction and sign it with `signer`.

    // TODO: Send transaction to the network, using Rust SDK.

    Ok("ok".to_string())
}
