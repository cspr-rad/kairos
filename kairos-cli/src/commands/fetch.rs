use clap::{Parser, ValueEnum};
use reqwest::Url;

use crate::client;
use crate::error::CliError;
use kairos_data::transaction::{Transaction, TransactionFilter};

use chrono::NaiveDateTime;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, short, value_name = "PUBLIC_KEY_HEX")]
    sender: Option<String>,
    #[arg(long, short, value_name = "ISO8601_TIMESTAMP")]
    min_timestamp: Option<NaiveDateTime>,
    #[arg(long, short, value_name = "ISO8601_TIMESTAMP")]
    max_timestamp: Option<NaiveDateTime>,
    #[arg(long, short, value_name = "NUM_MOTES")]
    min_amount: Option<u64>,
    #[arg(long, short, value_name = "NUM_MOTES")]
    max_amount: Option<u64>,
    #[arg(long, short, value_name = "PUBLIC_KEY_HEX")]
    recipient: Option<String>,
    #[arg(long, short, value_name = "TRANSACTION_TYPE", value_enum)]
    transaction_type: Option<TransactionType>,
}

#[derive(ValueEnum, Debug, Clone)] // ArgEnum here
pub enum TransactionType {
    Deposit,
    Transfer,
    Withdrawal,
}

impl From<TransactionType> for Transaction {
    fn from(t: TransactionType) -> Transaction {
        match t {
            TransactionType::Deposit => Transaction::Deposit,
            TransactionType::Transfer => Transaction::Transfer,
            TransactionType::Withdrawal => Transaction::Withdrawal,
        }
    }
}

pub fn run(
    Args {
        sender,
        min_timestamp,
        max_timestamp,
        min_amount,
        max_amount,
        recipient,
        transaction_type,
    }: Args,
    kairos_server_address: Url,
) -> Result<String, CliError> {
    let transaction_filter = TransactionFilter {
        sender,
        min_timestamp,
        max_timestamp,
        min_amount,
        max_amount,
        recipient,
        transaction_type: transaction_type.map(Into::into),
    };
    let transactions = client::fetch(&kairos_server_address, &transaction_filter)
        .map_err(Into::<CliError>::into)?;
    serde_json::to_string_pretty(&transactions).map_err(Into::into)
}
