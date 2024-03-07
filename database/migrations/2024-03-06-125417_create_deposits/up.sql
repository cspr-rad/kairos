-- Your SQL goes here
CREATE TABLE IF NOT EXISTS deposits (
    account VARCHAR(32) NOT NULL,
    amount numeric NOT NULL,
    processed BOOLEAN DEFAULT FALSE NOT NULL,
    id TIMESTAMP WITHOUT TIME ZONE PRIMARY KEY
);