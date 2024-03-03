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

fn main() {
    println!("Hello, world!");
}
