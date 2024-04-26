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
    DigestHasher, KeyHash, NodeHash, Transaction, TrieRoot,
};
use sha2::Sha256;

use types::{DemoCircuitInput, Operation, Value};

risc0_zkvm::guest::entry!(main);

fn verify_snapshot_and_compute_root(
    batch: &[Operation],
    snapshot: Snapshot<[u8; 8]>,
    old_root_hash: TrieRoot<NodeHash>
) -> TrieRoot<NodeHash>{
    assert_eq!(
        old_root_hash,
        snapshot
            .calc_root_hash(&mut DigestHasher::<Sha256>::default())
            .unwrap()
    );

    let mut txn = Transaction::from_snapshot(&snapshot).unwrap();

    for op in batch{
        trie_op(op, &mut txn);
    }

    let root_hash = txn
        .calc_root_hash(&mut DigestHasher::<Sha256>::default())
        .unwrap();

    root_hash
}

fn trie_op<S: Store<Value>>(
    op: &Operation,
    txn: &mut Transaction<S, Value>,
) -> (Option<Value>, Option<Value>) {
    match op {
        Operation::Insert(key, value) => {
            txn.insert(key, *value).unwrap();
            (None, Some(*value))
        }
        Operation::EntryInsert(key, value) => match txn.entry(key).unwrap() {
            kairos_trie::Entry::Occupied(mut o) => {
                let old = *o.get();
                o.insert(*value);
                (Some(old), Some(*value))
            }
            kairos_trie::Entry::Vacant(v) => {
                let new = v.insert(*value);
                (None, Some(*new))
            }
            kairos_trie::Entry::VacantEmptyTrie(v) => {
                let new = v.insert(*value);
                (None, Some(*new))
            }
        },
        Operation::EntryAndModifyOrInsert(key, value) => {
            let entry = txn.entry(key).unwrap();
            let mut old = None;
            let new = entry
                .and_modify(|v| {
                    old = Some(*v);
                    *v = *value;
                })
                .or_insert(*value);

            assert_eq!(new, value);

            (old, Some(*new))
        }
        Operation::EntryOrInsert(key, value) => {
            let mut old = None;
            let new = txn
                .entry(key)
                .unwrap()
                .and_modify(|v| old = Some(*v))
                .or_insert(*value);

            (old, Some(*new))
        }
        Operation::Get(key) => {
            let old = txn.get(key).unwrap().copied();
            (old, old)
        }
        Operation::EntryGet(key) => {
            let old = txn.entry(key).unwrap().get().copied();
            (old, old)
        }
    }
}

fn main() {
    // TODO: Implement your guest code here

    // read the input
    let input: DemoCircuitInput = env::read();

    // TODO: do something with the input
    let output: TrieRoot<NodeHash> = verify_snapshot_and_compute_root(&input.batch, input.snapshot, input.new_root_hash);
    // write public output to the journal
    env::commit(&output);
}
