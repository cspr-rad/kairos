// @generated automatically by Diesel CLI.

diesel::table! {
    deposits (id) {
        #[max_length = 32]
        account -> Varchar,
        amount -> Numeric,
        processed -> Bool,
        id -> Timestamp,
    }
}

diesel::table! {
    transfers (id) {
        #[max_length = 32]
        sender -> Varchar,
        #[max_length = 32]
        recipient -> Varchar,
        amount -> Numeric,
        id -> Timestamp,
        sig -> Bytea,
        processed -> Bool,
        nonce -> BigInt,
    }
}

diesel::table! {
    withdrawals (id) {
        #[max_length = 32]
        account -> Varchar,
        amount -> Numeric,
        processed -> Bool,
        id -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    deposits,
    transfers,
    withdrawals,
);
