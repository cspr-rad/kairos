use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, Selectable, SelectableHelper};
use diesel::RunQueryDsl;
use kairos_risc0_types::{ToBigDecimal, Withdrawal};
use chrono::{Utc, NaiveDateTime};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use deadpool_diesel::postgres::Pool;

use crate::database::delta_tree_schema::withdrawals;
use crate::database::errors;
use crate::database::delta_tree_schema as schema;

#[derive(Serialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::withdrawals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WithdrawalModel {
    pub account: String,
    pub amount: BigDecimal,
    pub processed: bool,
    pub timestamp: NaiveDateTime,
}

impl From<Withdrawal> for WithdrawalModel {
    fn from(deposit: Withdrawal) -> Self {
        WithdrawalModel {
            account: deposit.account.to_string(),
            amount: deposit.amount.to_big_decimal(),
            processed: deposit.processed,
            timestamp: Utc::now().naive_utc(),
        }
    }
}

#[derive(Deserialize)]
pub struct DepositFilter {
    account: Option<String>,
    processed: Option<bool>,
}

pub async fn insert(pool: Pool, new_deposit: Withdrawal) -> Result<WithdrawalModel, errors::DatabaseError> {
    let conn = pool.get().await?;
    let deposit_model = WithdrawalModel::from(new_deposit);
    let res = conn
            .interact(|conn| {
                diesel::insert_into(withdrawals::table)
                    .values(deposit_model)
                    .get_result::<WithdrawalModel>(conn)
            })
        .await??;
    Ok(res)
}

// TODO - add get function which just retrieves by ID

pub async fn get_all(pool: Pool, filter: DepositFilter) -> Result<Vec<WithdrawalModel>, errors::DatabaseError> {
    let conn = pool.get().await?;
    let res = conn
        .interact(move |conn| {
            let mut query = withdrawals::table.into_boxed::<diesel::pg::Pg>();

            if let Some(account) = filter.account {
                query = query.filter(withdrawals::account.eq(account));
            }

            if let Some(processed) = filter.processed {
                query = query.filter(withdrawals::processed.eq(processed))
            }

            query.select(WithdrawalModel::as_select()).load::<WithdrawalModel>(conn)
        })
        .await??;
    Ok(res)
}
