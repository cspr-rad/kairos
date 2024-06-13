// @generated automatically by Diesel CLI.

diesel::table! {
    transactions (timestamp, amount, public_key) {
        timestamp -> Timestamp,
        public_key -> Varchar,
        nonce -> Nullable<Int8>,
        trx -> Int2,
        amount -> Int8,
        recipient -> Nullable<Varchar>,
    }
}
