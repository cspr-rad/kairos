mod utils;
use std::{collections::HashMap, rc::Rc};

use proptest::prelude::*;

use kairos_trie::{
    stored::{memory_db::MemoryDb, merkle::SnapshotBuilder},
    KeyHash, Transaction, TrieRoot,
};
use utils::operations::*;

fn end_to_end_entry_ops(batches: Vec<Vec<Operation>>) {
    // The persistent backing, likely rocksdb
    let db = Rc::new(MemoryDb::<[u8; 8]>::empty());

    // An empty trie root
    let mut prior_root_hash = TrieRoot::default();

    // used as a reference for trie behavior
    let mut hash_map = HashMap::new();

    for batch in batches.iter() {
        eprintln!("Batch size: {}", batch.len());
        // We build a snapshot on the server.
        let (new_root_hash, snapshot) =
            run_against_snapshot_builder(batch, prior_root_hash, db.clone(), &mut hash_map);

        // We verify the snapshot in a zkVM
        run_against_snapshot(batch, snapshot, new_root_hash, prior_root_hash);

        // After a batch is verified in an on chain zkVM the contract would update's its root hash
        prior_root_hash = new_root_hash;
    }

    // After all batches are applied, the trie and the hashmap should be in sync
    let txn = Transaction::from_snapshot_builder(
        SnapshotBuilder::<_, [u8; 8]>::empty(db).with_trie_root_hash(prior_root_hash),
    );

    // Check that the trie and the hashmap are in sync
    for (k, v) in hash_map.iter() {
        let ret_v = txn.get(k).unwrap().unwrap();
        assert_eq!(v, ret_v);
    }
}

proptest! {
    #[test]
    fn prop_end_to_end_entry_ops(
        batches in arb_batches(1..5000usize, 1..100_000usize, 1000, 10_000)) {
        end_to_end_entry_ops(batches);
    }
}

#[test]
fn leaf_prefix_insert() {
    let failed = vec![vec![
        Operation::Insert(KeyHash([1, 0, 0, 0, 0, 0, 0, 0]), 0u64.to_le_bytes()),
        Operation::Insert(KeyHash([1, 0, 0, 0, 0, 0, 0, 1]), 0u64.to_le_bytes()),
    ]];

    end_to_end_entry_ops(failed);
}

#[test]
fn leaf_prefix_insert_at_root() {
    let failed = vec![vec![
        Operation::Insert(KeyHash([1, 0, 0, 0, 0, 0, 0, 0]), 0u64.to_le_bytes()),
        Operation::Insert(KeyHash([1, 0, 0, 0, 0, 0, 0, 1]), 0u64.to_le_bytes()),
        Operation::Insert(KeyHash([0, 0, 0, 0, 0, 0, 0, 0]), 0u64.to_le_bytes()),
    ]];

    end_to_end_entry_ops(failed);
}

#[test]
fn leaf_prefix_insert_entry_insert_at_root() {
    let failed = vec![vec![
        Operation::Insert(KeyHash([1, 0, 0, 0, 0, 0, 0, 0]), 0u64.to_le_bytes()),
        Operation::Insert(KeyHash([1, 0, 0, 0, 0, 0, 0, 1]), 0u64.to_le_bytes()),
        Operation::EntryInsert(KeyHash([0, 0, 0, 0, 0, 0, 0, 0]), 0u64.to_le_bytes()),
    ]];

    end_to_end_entry_ops(failed);
}

#[test]
fn leaf_prefix_entry_insert() {
    let failed = vec![vec![
        Operation::EntryInsert(KeyHash([1, 0, 0, 0, 0, 0, 0, 0]), 0u64.to_le_bytes()),
        Operation::EntryInsert(KeyHash([1, 0, 0, 0, 0, 0, 0, 1]), 0u64.to_le_bytes()),
    ]];

    end_to_end_entry_ops(failed);
}

#[test]
fn leaf_prefix_entry_or_insert() {
    let failed = vec![vec![
        Operation::EntryOrInsert(KeyHash([1, 0, 0, 0, 0, 0, 0, 0]), 0u64.to_le_bytes()),
        Operation::EntryOrInsert(KeyHash([1, 0, 0, 0, 0, 0, 0, 1]), 0u64.to_le_bytes()),
    ]];

    end_to_end_entry_ops(failed);
}
