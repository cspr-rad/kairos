use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, Selectable, SelectableHelper};
use diesel::RunQueryDsl;
use kairos_risc0_types::{ToBigDecimal, Deposit};
use chrono::{Utc, NaiveDateTime};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use deadpool_diesel::postgres::Pool;

use crate::database::delta_tree_schema::deposits;
use crate::database::errors;
use crate::database::delta_tree_schema as schema;

use kairos_risc0_types;

#[derive(Serialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::deposits)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DepositModel {
    pub account: String,
    pub amount: BigDecimal,
    pub processed: bool,
    pub timestamp: NaiveDateTime,
}

impl From<Deposit> for DepositModel {
    fn from(deposit: Deposit) -> Self {
        DepositModel {
            account: deposit.account.to_string(),
            amount: deposit.amount.to_big_decimal(),
            processed: deposit.processed,
            timestamp: Utc::now().naive_utc(),
        }
    }
}

impl Into<Deposit> for DepositModel {
    fn into(self) -> Deposit {
        Deposit {
            account:kairos_risc0_types::Key::Account(kairos_risc0_types::AccountHash::from_formatted_str(&self.account).unwrap()),
            amount:kairos_risc0_types::U512::from_dec_str(&self.amount.to_string()).unwrap(),
            timestamp: None,
            processed: self.processed,
        }
    }
}

#[derive(Deserialize)]
pub struct DepositFilter {
    pub account: Option<String>,
    pub processed: Option<bool>,
}

pub async fn insert(pool: Pool, new_deposit: Deposit) -> Result<DepositModel, errors::DatabaseError> {
    let conn = pool.get().await?;
    let deposit_model = DepositModel::from(new_deposit);
    let res = conn
            .interact(|conn| {
                diesel::insert_into(deposits::table)
                    .values(deposit_model)
                    .get_result::<DepositModel>(conn)
            })
        .await??;
    Ok(res)
}

// TODO - add get function which just retrieves by ID

pub async fn get_all(pool: Pool, filter: DepositFilter) -> Result<Vec<DepositModel>, errors::DatabaseError> {
    let conn = pool.get().await?;
    let res = conn
        .interact(move |conn| {
            let mut query = deposits::table.into_boxed::<diesel::pg::Pg>();

            if let Some(account) = filter.account {
                query = query.filter(deposits::account.eq(account));
            }

            if let Some(processed) = filter.processed {
                query = query.filter(deposits::processed.eq(processed))
            }

            query.select(DepositModel::as_select()).load::<DepositModel>(conn)
        })
        .await??;
    Ok(res)
}
