use crate::schema::transactions;
use bigdecimal::BigDecimal;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use hex;
use serde::{Deserialize, Serialize};

use kairos_circuit_logic::transactions::{
    KairosTransaction, L1Deposit, Signed, Transfer, Withdraw,
};

#[derive(diesel_derive_enum::DbEnum, Debug, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::Transaction"]
pub enum Transaction {
    Deposit,
    Transfer,
    Withdrawal,
}

#[derive(Queryable, Debug, Identifiable, Insertable, Serialize, Selectable)]
#[diesel(primary_key(timestamp, amount, public_key))]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transactions {
    pub timestamp: NaiveDateTime,
    pub public_key: String,
    pub nonce: Option<BigDecimal>,
    pub trx: Transaction,
    pub amount: BigDecimal,
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
    pool: &crate::Pool,
    filter: TransactionFilter,
) -> Result<Vec<Transactions>, crate::errors::DBError> {
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
                query = query.filter(transactions::amount.ge(BigDecimal::from(min_amount)));
            }
            if let Some(max_amount) = filter.max_amount {
                query = query.filter(transactions::amount.le(BigDecimal::from(max_amount)));
            }
            if let Some(recipient) = filter.recipient {
                query = query.filter(transactions::recipient.eq(recipient));
            }

            query
                .select(Transactions::as_select())
                .limit(500)
                .load::<Transactions>(conn)
        })
        .await??;
    Ok(res)
}

pub async fn insert(
    pool: &crate::Pool,
    kairos_trx: KairosTransaction,
) -> Result<Transactions, crate::errors::DBError> {
    let trx = Transactions::from(kairos_trx);
    let conn = pool.get().await?;
    let res = conn
        .interact(|conn| {
            diesel::insert_into(transactions::table)
                .values(trx)
                .get_result::<Transactions>(conn)
        })
        .await??;
    Ok(res)
}

impl From<KairosTransaction> for Transactions {
    fn from(tx: KairosTransaction) -> Self {
        match tx {
            KairosTransaction::Transfer(signed_transfer) => Transactions::from(signed_transfer),
            KairosTransaction::Withdraw(signed_withdraw) => Transactions::from(signed_withdraw),
            KairosTransaction::Deposit(l1_deposit) => Transactions::from(l1_deposit),
        }
    }
}

impl From<Signed<Transfer>> for Transactions {
    fn from(signed_transfer: Signed<Transfer>) -> Self {
        Transactions {
            timestamp: Utc::now().naive_utc(),
            public_key: hex::encode(&signed_transfer.public_key),
            nonce: Some(BigDecimal::from(signed_transfer.nonce)),
            trx: Transaction::Transfer,
            amount: BigDecimal::from(signed_transfer.transaction.amount),
            recipient: Some(hex::encode(&signed_transfer.transaction.recipient)),
        }
    }
}

impl From<Signed<Withdraw>> for Transactions {
    fn from(signed_withdraw: Signed<Withdraw>) -> Self {
        Transactions {
            timestamp: Utc::now().naive_utc(),
            public_key: hex::encode(&signed_withdraw.public_key),
            nonce: Some(BigDecimal::from(signed_withdraw.nonce)),
            trx: Transaction::Withdrawal,
            amount: BigDecimal::from(signed_withdraw.transaction.amount),
            recipient: None,
        }
    }
}

impl From<L1Deposit> for Transactions {
    fn from(deposit: L1Deposit) -> Self {
        Transactions {
            timestamp: Utc::now().naive_utc(),
            public_key: hex::encode(&deposit.recipient),
            nonce: None,
            trx: Transaction::Deposit,
            amount: BigDecimal::from(deposit.amount),
            recipient: Some(hex::encode(&deposit.recipient)),
        }
    }
}
