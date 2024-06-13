use crate::schema::transactions;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use hex;
use serde::{Deserialize, Serialize};

use kairos_circuit_logic::transactions::{
    KairosTransaction, L1Deposit, Signed, Transfer, Withdraw,
};

const TRANSFER_TRX: i16 = 1;
const DEPOSIT_TRX: i16 = 2;
const WITHDRAW_TRX: i16 = 3;

#[derive(Queryable, Debug, Identifiable, Insertable, Serialize, Selectable)]
#[diesel(primary_key(timestamp, amount, public_key))]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub timestamp: NaiveDateTime,
    pub public_key: String,
    pub nonce: Option<i64>,
    pub trx: i16,
    pub amount: i64,
    pub recipient: Option<String>,
}

#[derive(Deserialize)]
pub struct TransactionFilter {
    pub sender: Option<String>,
    pub min_timestamp: Option<NaiveDateTime>,
    pub max_timestamp: Option<NaiveDateTime>,
    pub min_amount: Option<i64>,
    pub max_amount: Option<i64>,
    pub recipient: Option<String>,
}

pub async fn get(
    pool: crate::Pool,
    filter: TransactionFilter,
) -> Result<Vec<Transaction>, crate::errors::DBError> {
    let conn = pool.get().await?;
    let res = conn
        .interact(move |conn| {
            let mut query = transactions::table.into_boxed::<diesel::pg::Pg>();

            if let Some(sender) = filter.sender {
                query = query.filter(transactions::public_key.eq(sender));
            }
            if let Some(min_timestamp) = filter.min_timestamp {
                query = query.filter(transactions::timestamp.ge(min_timestamp));
            }
            if let Some(max_timestamp) = filter.max_timestamp {
                query = query.filter(transactions::timestamp.le(max_timestamp));
            }
            if let Some(min_amount) = filter.min_amount {
                query = query.filter(transactions::amount.ge(min_amount));
            }
            if let Some(max_amount) = filter.max_amount {
                query = query.filter(transactions::amount.le(max_amount));
            }
            if let Some(recipient) = filter.recipient {
                query = query.filter(transactions::recipient.eq(recipient));
            }

            query
                .select(Transaction::as_select())
                .limit(500)
                .load::<Transaction>(conn)
        })
        .await??;
    Ok(res)
}

pub async fn insert(
    pool: crate::Pool,
    kairos_trx: KairosTransaction,
) -> Result<Transaction, crate::errors::DBError> {
    let trx = Transaction::from(kairos_trx);
    let conn = pool.get().await?;
    let res = conn
        .interact(|conn| {
            diesel::insert_into(transactions::table)
                .values(trx)
                .get_result::<Transaction>(conn)
        })
        .await??;
    Ok(res)
}

impl From<KairosTransaction> for Transaction {
    fn from(tx: KairosTransaction) -> Self {
        match tx {
            KairosTransaction::Transfer(signed_transfer) => Transaction::from(signed_transfer),
            KairosTransaction::Withdraw(signed_withdraw) => Transaction::from(signed_withdraw),
            KairosTransaction::Deposit(l1_deposit) => Transaction::from(l1_deposit),
        }
    }
}

impl From<Signed<Transfer>> for Transaction {
    fn from(signed_transfer: Signed<Transfer>) -> Self {
        Transaction {
            timestamp: Utc::now().naive_utc(),
            public_key: hex::encode(&signed_transfer.public_key),
            nonce: Some(signed_transfer.nonce as i64),
            trx: TRANSFER_TRX,
            amount: signed_transfer.transaction.amount as i64,
            recipient: Some(hex::encode(&signed_transfer.transaction.recipient)),
        }
    }
}

impl From<Signed<Withdraw>> for Transaction {
    fn from(signed_withdraw: Signed<Withdraw>) -> Self {
        Transaction {
            timestamp: Utc::now().naive_utc(),
            public_key: hex::encode(&signed_withdraw.public_key),
            nonce: Some(signed_withdraw.nonce as i64),
            trx: WITHDRAW_TRX,
            amount: signed_withdraw.transaction.amount as i64,
            recipient: None,
        }
    }
}

impl From<L1Deposit> for Transaction {
    fn from(deposit: L1Deposit) -> Self {
        Transaction {
            timestamp: Utc::now().naive_utc(),
            public_key: hex::encode(&deposit.recipient),
            nonce: None,
            trx: DEPOSIT_TRX,
            amount: deposit.amount as i64,
            recipient: Some(hex::encode(&deposit.recipient)),
        }
    }
}
