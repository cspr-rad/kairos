// This service will query the L1 for deposit events and update balances accordingly.
// Every transaction that is recorded will be included in the next Batch.
// Transactions can be submitted directly to the L2 service via the L2 client
// Insufficient L2 Balance at time of submission will lead to rejection
// Invalid signature will lead to immediate rejection
// Valid transactions will be included in a Batch (see kairos-risc0-types/batch)

// The proof 
// Verify Signatures and Apply state transitions (Balances in => Balances out)
// Hash the Batch and include it in the Tornado Tree
// Write the new State and Root to the journal
// Submit the new Balances, Root and Proof to the L1
// The L1 will update the affected account Balances and Root if the Proof is valid


// New Transfers are inserted into a SQLite DB and so are Deposit Events
// Every Transaction (Transfer, Deposit, Withdrawal) that is recorded on the L2 
// is assigned a status (e.g. processed=False).

// With the Transaction was processed/ successfully included in a batch the status is updated to processed = True.

// Store the "Accounting" Struct locally - temporary and inefficient solution.
// Ideally submitting affected Balances to the L1 is not necessary.

fn main() {
    todo!("
        - Implement the L1 listener
        - Check incoming Transfer's validity according to L2 state
        - Store Transactions with processed=false & update the local L2 Balance
        - Rollup Transactions and utilize the circuit to produce a Proof 
        - Update L1 State accordingly
    ")
}
