// @generated automatically by Diesel CLI.

diesel::table! {
    batches (timestamp) {
        deposits -> Bytea,
        transfers -> Bytea,
        withdrawals -> Bytea,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    deposits (timestamp) {
        #[max_length = 32]
        account -> Varchar,
        amount -> Numeric,
        processed -> Bool,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    transfers (timestamp) {
        sender -> Varchar,
        recipient -> Varchar,
        amount -> Numeric,
        timestamp -> Timestamp,
        sig -> Bytea,
        processed -> Bool,
        nonce -> Numeric,
    }
}

diesel::table! {
    withdrawals (timestamp) {
        account -> Varchar,
        amount -> Numeric,
        processed -> Bool,
        timestamp -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    batches,
    deposits,
    transfers,
    withdrawals,
);
