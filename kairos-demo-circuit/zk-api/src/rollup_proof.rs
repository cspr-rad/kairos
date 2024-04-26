use methods::{
    DEMO_CIRCUIT_ELF, DEMO_CIRCUIT_ID
};
use risc0_zkvm::{default_prover, ExecutorEnv};
use sha2::Sha256;

use kairos_trie::{
    stored::{memory_db::MemoryDb, merkle::SnapshotBuilder, DatabaseSet}, KeyHash, NodeHash, Transaction, TrieRoot,
    stored::{
        merkle::Snapshot,
        Store,
    },
    DigestHasher
};

use std::{
    collections::{hash_map, HashMap},
    rc::Rc,
};
use types::{Operation, Value};
use types::DemoCircuitInput;

