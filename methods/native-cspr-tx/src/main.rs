#![no_main]
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);
use kairos_risc0_types::{CircuitArgs, CircuitJournal, HashableStruct, MockAccounting, MockLayerTwoStorage, TornadoTree, Transaction, TransactionHistory};

pub fn main() {
    let mut inputs: CircuitArgs = env::read();
    let mut circuit_journal = CircuitJournal{
        input: inputs.clone(),
        output: None
    };
    let transactions: TransactionHistory = inputs.clone().mock_storage.transactions;
    let mut balances: MockAccounting = inputs.clone().mock_storage.balances;
    let mut tree: TornadoTree = inputs.tornado;

    
    // mutate the state
    for transaction in transactions.clone().transactions{
        match transaction{
            Transaction::Deposit { account, amount, processed, id } => {
                if balances.balances.contains_key(&account){
                    balances.balances.insert(account.clone(), balances.balances.get(&account).expect("Expected Balance under Key in Mock Storage!") + amount);
                }
                else{
                    balances.balances.insert(account.clone(), amount);
                }
            },
            Transaction::Transfer { sender, recipient, amount, signature, processed, nonce } => {
                // todo: verify signature
                // !!! IMPORTANT !!!
                // todo!("Must implement signature Verification!");
                // sender must exist
                assert!(balances.balances.contains_key(&sender));                
                let sender_balance = balances.balances.get(&sender).expect("Sender has no Balance Value!");
                // sender balance must be at least amount
                assert!(sender_balance >= &amount);

                // decrease sender balance by amount
                balances.balances.insert(sender.clone(), balances.balances.get(&sender).expect("Expected Sender Balance under Key in Mock Storage!") - amount);
                // if recipient exists, increase the balance
                if balances.balances.contains_key(&recipient){
                    // increase recipient balance by amount
                    balances.balances.insert(recipient.clone(), balances.balances.get(&recipient).expect("Expected Sender Balance under Key in Mock Storage!") + amount);
                }
                // otherwise create funded recipient account
                else{
                    balances.balances.insert(recipient.clone(), amount);
                }
            },
            // Withdrawal is yet to be implemented.
            Transaction::Withdrawal { account, amount, processed, id } => {
                todo!("Implement Withdrawals!");
            }
        }
    }

    // hash Balances and add a new leaf to the Tree
    // todo: construct merkle proofs so that Client can verify the last n leafs (where n is the root history length)
    let new_leaf: Vec<u8> = balances.hash();
    tree.add_leaf(new_leaf);
    circuit_journal.output = Some(CircuitArgs{
        tornado: tree,
        mock_storage: MockLayerTwoStorage{
            balances: balances,
            transactions: transactions
        }
    });
    env::commit(&circuit_journal);
}
