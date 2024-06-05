use std::path::PathBuf;

use clap::Args;

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
pub struct ContractHashArg {
    #[arg(id = "contract-hash", long, short = 'h', value_name = "CONTRACT_HASH")]
    pub field: String,
}
