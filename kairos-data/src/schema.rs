// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "transaction"))]
    pub struct Transaction;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Transaction;

    transactions (timestamp, amount, public_key) {
        timestamp -> Timestamp,
        public_key -> Varchar,
        nonce -> Nullable<Numeric>,
        trx -> Transaction,
        amount -> Numeric,
        recipient -> Nullable<Varchar>,
    }
}
