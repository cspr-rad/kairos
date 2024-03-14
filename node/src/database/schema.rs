// @generated automatically by Diesel CLI.

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
        #[max_length = 32]
        sender -> Varchar,
        #[max_length = 32]
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
        #[max_length = 32]
        account -> Varchar,
        amount -> Numeric,
        processed -> Bool,
        timestamp -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    deposits,
    transfers,
    withdrawals,
);
