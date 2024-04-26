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

use types::{DemoCircuitInput, Transaction};

risc0_zkvm::guest::entry!(main);

fn run_against_snapshot_and_return_root(
    batch: &[Transaction],
    snapshot: Snapshot<[u8; 8]>,
    old_root_hash: TrieRoot<NodeHash>
) -> TrieRoot<NodeHash>{
    assert_eq!(
        old_root_hash,
        snapshot
            .calc_root_hash(&mut DigestHasher::<Sha256>::default())
            .unwrap()
    );
    let mut txn = kairos_trie::Transaction::from_snapshot(&snapshot).unwrap();
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
