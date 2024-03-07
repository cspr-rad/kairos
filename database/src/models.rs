use kairos_risc0_types::{Deposit, Withdrawal, Transfer, Key, U512};
use bigdecimal::BigDecimal;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use std::str::FromStr;
use crate::schema::*;

#[derive(Insertable, Queryable)]
#[table_name = "deposits"]
pub struct DepositRow {
    pub account: String,
    pub amount: BigDecimal,
    pub processed: bool,
    pub id: NaiveDateTime,
}

impl From<Deposit> for DepositRow {
    fn from(item: Deposit) -> Self {
        let timestamp = match item.timestamp {
            Some(dt) => dt,
            None => Utc::now().naive_utc(),
        };
        DepositRow {
            account: item.account.to_string(),
            amount: BigDecimal::from_str(&item.amount.to_string()).expect("Failed to parse to BigDecimal"),
            processed: item.processed,
            id: timestamp,
        }
    }
}

#[derive(Insertable, Queryable)]
#[table_name = "withdrawals"]
pub struct WithdrawalRow {
    pub account: String,
    pub amount: BigDecimal,
    pub processed: bool,
    pub id: NaiveDateTime,
}

impl From<Withdrawal> for WithdrawalRow {
    fn from(item: Withdrawal) -> Self {
        WithdrawalRow {
            account: item.account.to_string(),
            amount: BigDecimal::from_str(&item.amount.to_string()).expect("Failed to parse to BigDecimal"),
            processed: item.processed,
            id: item.timestamp,
        }
    }
}

#[derive(Insertable, Queryable)]
#[table_name = "transfers"]
pub struct TransferRow {
    pub sender: String,
    pub recipient: String,
    pub amount: BigDecimal,
    pub id: NaiveDateTime,
    pub sig: Vec<u8>,
    pub processed: bool,
    pub nonce: i64
}

impl From<Transfer> for TransferRow {
    fn from(item: Transfer) -> Self {
        TransferRow {
            sender: item.sender.to_string(),
            recipient: item.recipient.to_string(),
            amount: BigDecimal::from_str(&item.amount.to_string()).expect("Failed to parse to BigDecimal"),
            id: item.timestamp,
            sig: item.signature,
            processed: item.processed,
            nonce: item.nonce as i64,
        }
    }
}