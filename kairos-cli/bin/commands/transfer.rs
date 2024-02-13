use crate::common::args::{AmountArg, PrivateKeyPathArg};
use crate::crypto::public_key::CasperPublicKey;
use crate::crypto::signer::CasperSigner;
use crate::error::CliError;
use crate::utils::parse_hex_string;

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
    let _recipient = CasperPublicKey::from_bytes(args.recipient.as_ref())?;
    let _amount: u64 = args.amount.field;
    let _signer = CasperSigner::from_key_pathbuf(args.private_key_path.field)?;

    // TODO: Create transaction and sign it with `signer`.

    // TODO: Send transaction to the network, using Rust SDK.

    Ok("ok".to_string())
}
