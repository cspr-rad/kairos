#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std]  // std support is experimental
use risc0_zkvm::guest::env;
use kairos_trie::{
    stored::{
        memory_db::MemoryDb,
        merkle::{Snapshot, SnapshotBuilder},
        Store,
    },
    DigestHasher, KeyHash, NodeHash, TrieRoot,
};
use sha2::Sha256;

use types::{DemoCircuitInput, transactions, verification_logic};
use kairos_common_types;
risc0_zkvm::guest::entry!(main);

fn run_against_snapshot_and_return_root(
    batch: &[kairos_common_types::transactions::Signed<transactions::Transaction>],
    snapshot: Snapshot<verification_logic::Account>,
    old_root_hash: TrieRoot<NodeHash>
) -> TrieRoot<NodeHash>{
    assert_eq!(
        old_root_hash,
        snapshot
            .calc_root_hash(&mut DigestHasher::<Sha256>::default())
            .unwrap()
    );
    let mut txn = kairos_trie::Transaction::from_snapshot(&snapshot).unwrap();
    let mut batch_state = verification_logic::BatchState::new(txn);
    for tx in batch{
        batch_state.execute_transaction(tx.clone());
    }
    let txn = kairos_trie::Transaction::from_snapshot(&snapshot).unwrap();
    let root_hash = txn
        .calc_root_hash(&mut DigestHasher::<Sha256>::default())
        .unwrap();

    root_hash
}

fn main() {
    let input: DemoCircuitInput = env::read();
    let output: TrieRoot<NodeHash> = run_against_snapshot_and_return_root(&input.batch, input.snapshot, input.new_root_hash);
    // write public output to the journal
    env::commit(&output);
}
