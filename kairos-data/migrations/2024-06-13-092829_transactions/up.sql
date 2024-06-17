CREATE TYPE transaction AS ENUM ('Deposit', 'Transfer', 'Withdrawal');
CREATE TABLE transactions (
    "timestamp" timestamp DEFAULT CURRENT_TIMESTAMP,
    public_key varchar NOT NULL,
    nonce numeric,
    trx transaction NOT NULL,
    amount numeric NOT NULL,
    recipient varchar,
    PRIMARY KEY ("timestamp", amount, public_key)
);
