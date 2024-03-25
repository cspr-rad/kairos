use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, Selectable, SelectableHelper};
use diesel::RunQueryDsl;
use bincode;
use chrono::{Utc, NaiveDateTime};
use serde::{Deserialize, Serialize};
use deadpool_diesel::postgres::Pool;

use crate::database::delta_tree_schema::batches;
use crate::database::errors;
use crate::database::delta_tree_schema as schema;

use kairos_risc0_types::TransactionBatch;
use kairos_risc0_types;


#[derive(Serialize, Selectable, Insertable, Queryable)]
#[diesel(table_name = schema::batches)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BatchModel {
    pub deposits: Vec<u8>,    
    pub transfers: Vec<u8>,
    pub withdrawals: Vec<u8>,
    pub timestamp: NaiveDateTime,
}

impl From<TransactionBatch> for BatchModel {
    fn from(batch: TransactionBatch) -> Self {
        BatchModel {
            deposits: bincode::serialize(&batch.deposits).unwrap(),
            withdrawals: bincode::serialize(&batch.withdrawals).unwrap(),
            transfers: bincode::serialize(&batch.transfers).unwrap(),
            timestamp: Utc::now().naive_utc(),
        }
    }
}

pub async fn insert(pool: Pool, new_batch: TransactionBatch) -> Result<BatchModel, errors::DatabaseError> {
    let conn = pool.get().await?;
    let deposit_model = BatchModel::from(new_batch);
    let res = conn
            .interact(|conn| {
                diesel::insert_into(batches::table)
                    .values(deposit_model)
                    .get_result::<BatchModel>(conn)
            })
        .await??;
    Ok(res)
}