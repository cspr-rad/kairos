-- Your SQL goes here
CREATE TABLE IF NOT EXISTS "batches" (
    deposits BYTEA NOT NULL,
    transfers BYTEA NOT NULL,
    withdrawals BYTEA NOT NULL,
    "timestamp" TIMESTAMP WITHOUT TIME ZONE PRIMARY KEY NOT NULL
)