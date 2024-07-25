use std::path::PathBuf;

use casper_client_types::bytesrepr::FromBytes;
use clap::Args;
use kairos_crypto::error::CryptoError;

use crate::utils::parse_hex_string;

#[derive(Args, Debug)]
pub struct AmountArg {
    #[arg(id = "amount", long, short, value_name = "NUM_MOTES")]
    pub field: u64,
}

#[derive(Args, Debug)]
pub struct PrivateKeyPathArg {
    #[arg(id = "private-key", long, short = 'k', value_name = "FILE_PATH")]
    pub field: PathBuf,
}

#[derive(Args, Debug)]
pub struct NonceArg {
    #[arg(id = "nonce", long, short, value_name = "NUM")]
    pub val: Option<u64>,
}

#[derive(Args, Debug)]
pub struct ContractHashArg {
    #[arg(id = "contract-hash", long, short = 'c', value_name = "CONTRACT_HASH")]
    pub field: Option<String>,
}

#[derive(Args, Debug)]
pub struct RecipientArg {
    #[arg(long, short, value_name = "PUBLIC_KEY", value_parser = parse_hex_string)]
    pub recipient: ::std::vec::Vec<u8>, // Absolute path is required here - see https://github.com/clap-rs/clap/issues/4626#issue-1528622454.
}

impl TryFrom<RecipientArg> for casper_client_types::PublicKey {
    type Error = CryptoError;

    fn try_from(arg: RecipientArg) -> Result<Self, CryptoError> {
        let (pk, _) = casper_client_types::PublicKey::from_bytes(&arg.recipient).map_err(|_| {
            CryptoError::FailedToParseKey {
                error: format!("invalid public key: {}", hex::encode(&arg.recipient)),
            }
        })?;

        Ok(pk)
    }
}

#[derive(Args, Debug)]
pub struct ChainNameArg {
    #[arg(
        id = "chain-name",
        long,
        value_name = "NAME",
        help = "Name of the chain, to avoid the deploy from being accidentally included in a different chain"
    )]
    pub field: Option<String>,
}
