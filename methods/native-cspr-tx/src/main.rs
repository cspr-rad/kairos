#![no_main]
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);
use kairos_risc0_types::{CircuitArgs, CircuitJournal, HashableStruct, MockAccounting, MockLayerTwoStorage, TornadoTree, Transaction, TransactionHistory, U512};

pub fn main() {
    let mut inputs: CircuitArgs = env::read();
    let mut circuit_journal = CircuitJournal{
        input: inputs.clone(),
        output: None
    };
    let transactions: TransactionHistory = inputs.clone().mock_storage.transactions;
    let mut balances: MockAccounting = inputs.clone().mock_storage.balances;
    let mut tree: TornadoTree = inputs.tornado;

    for (key, transaction) in transactions.transactions.iter() {
        match transaction {
            Transaction::Deposit { account, amount, .. } => {
                *balances.balances.entry(account.clone()).or_insert(U512::from(0)) += *amount;
            },
            Transaction::Transfer { sender, recipient, amount, .. } => {
                let sender_balance = balances.balances.entry(sender.clone()).or_insert(U512::from(0));
                assert!(*sender_balance >= *amount, "Sender balance is insufficient.");
                *sender_balance -= *amount;
                
                let recipient_balance = balances.balances.entry(recipient.clone()).or_insert(U512::from(0));
                *recipient_balance += *amount;
            },
            Transaction::Withdrawal { .. } => {
                todo!("Implement Withdrawals!");
            },
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
