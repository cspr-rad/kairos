-- Your SQL goes here
CREATE TABLE IF NOT EXISTS withdrawals (
    account VARCHAR NOT NULL,
    amount numeric NOT NULL,
    processed BOOLEAN DEFAULT FALSE NOT NULL,
    "timestamp" TIMESTAMP WITHOUT TIME ZONE PRIMARY KEY
);