-- Your SQL goes here
-- CREATE TYPE transaction AS ENUM ('Deposit', 'Transfer', 'Withdrawal');
CREATE TABLE transactions (
    "timestamp" timestamp DEFAULT CURRENT_TIMESTAMP,
    public_key varchar NOT NULL,
    nonce bigint,
    trx smallint NOT NULL,
    amount bigint NOT NULL,
    recipient varchar,
    PRIMARY KEY ("timestamp", amount, public_key)
);