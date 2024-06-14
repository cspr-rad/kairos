// @generated automatically by Diesel CLI.

diesel::table! {
    transactions (timestamp, amount, public_key) {
        timestamp -> Timestamp,
        public_key -> Varchar,
        nonce -> Nullable<Numeric>,
        trx -> Int2,
        amount -> Numeric,
        recipient -> Nullable<Varchar>,
    }
}
