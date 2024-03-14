use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, Selectable, SelectableHelper};
use diesel::RunQueryDsl;
use kairos_risc0_types::ToBigDecimal;
use chrono::{Utc, NaiveDateTime};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use deadpool_diesel::postgres::Pool;

use crate::database::schema::transfers;
use crate::database::errors;
use crate::database::schema;

use kairos_risc0_types::Transfer;

// Define struct for schema for transfers
#[derive(Serialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::transfers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransferModel {
    pub sender: String,
    pub recipient: String,
    pub amount: BigDecimal,
    pub timestamp: NaiveDateTime,
    pub sig: Vec<u8>,
    pub processed: bool,
    pub nonce: BigDecimal,
}

impl From<Transfer> for TransferModel {
    fn from(transfer: Transfer) -> Self {
        TransferModel {
            sender: transfer.sender.to_string(),
            recipient: transfer.recipient.to_string(),
            amount: transfer.amount.to_big_decimal(),
            timestamp: Utc::now().naive_utc(),
            sig: transfer.signature,
            processed: transfer.processed,
            nonce: transfer.nonce.into(),
        }
    }
}

#[derive(Deserialize)]
pub struct TransfersFilter {
    sender: Option<String>,
    recipient: Option<String>,
    processed: Option<bool>,
}

pub async fn insert(pool: Pool, new_transfer: Transfer) -> Result<TransferModel, errors::DatabaseError> {
    let conn = pool.get().await?;
    let transfer_model = TransferModel::from(new_transfer);
    let res = conn
            .interact(|conn| {
                diesel::insert_into(transfers::table)
                    .values(transfer_model)
                    .get_result::<TransferModel>(conn)
            })
        .await??;
    Ok(res)
}

// TODO - add get function which just retrieves transfer by ID

pub async fn get_all(pool: Pool, filter: TransfersFilter) -> Result<Vec<TransferModel>, errors::DatabaseError> {
    let conn = pool.get().await?;
    let res = conn
        .interact(move |conn| {
            let mut query = transfers::table.into_boxed::<diesel::pg::Pg>();

            if let Some(sender) = filter.sender {
                query = query.filter(transfers::sender.eq(sender));
            }

            if let Some(recipient) = filter.recipient {
                query = query.filter(transfers::recipient.eq(recipient));
            }

            if let Some(processed) = filter.processed {
                query = query.filter(transfers::processed.eq(processed))
            }

            query.select(TransferModel::as_select()).load::<TransferModel>(conn)
        })
        .await??;
    Ok(res)
}