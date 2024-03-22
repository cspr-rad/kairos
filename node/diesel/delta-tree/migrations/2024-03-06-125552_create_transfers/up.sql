-- Your SQL goes here
CREATE TABLE IF NOT EXISTS transfers (
    sender VARCHAR NOT NULL,
    recipient VARCHAR NOT NULL,
    amount numeric NOT NULL,
    "timestamp" TIMESTAMP WITHOUT TIME ZONE PRIMARY KEY,
    sig BYTEA NOT NULL, 
    processed BOOLEAN DEFAULT FALSE NOT NULL,
    nonce numeric NOT NULL
);