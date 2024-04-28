use circuits::{
    DEMO_CIRCUIT_ELF, DEMO_CIRCUIT_ID
};
use risc0_zkvm::{default_prover, ExecutorEnv};
use sha2::Sha256;

use kairos_trie::{
    stored::{memory_db::MemoryDb, merkle::SnapshotBuilder}, KeyHash, NodeHash, TrieRoot,
    stored::merkle::Snapshot,
    DigestHasher
};

use std::rc::Rc;

use types::{
    DemoCircuitInput, 
    transactions::Transaction, 
    transactions::Signed, 
    verification_logic::Account
};

pub fn prove(db: Rc<MemoryDb<Account>>, old_root_hash: TrieRoot<NodeHash>, operations: Vec<Signed<Transaction>>) -> risc0_zkvm::Receipt{
    let builder: SnapshotBuilder<Rc<MemoryDb<Account>>, Account> = SnapshotBuilder::empty(db).with_trie_root_hash(old_root_hash);
    let txn: kairos_trie::Transaction<SnapshotBuilder<Rc<MemoryDb<Account>>, Account>, Account> = kairos_trie::Transaction::from_snapshot_builder(builder);
    let new_root_hash: TrieRoot<NodeHash> = txn.commit(&mut DigestHasher::<Sha256>::default()).unwrap();
    let snapshot: Snapshot<Account> = txn.build_initial_snapshot();

    let circuit_input: DemoCircuitInput = DemoCircuitInput{
        batch: operations,
        snapshot: snapshot,
        new_root_hash: new_root_hash,
        old_root_hash: old_root_hash
    };

    let env = ExecutorEnv::builder()
        .write(&circuit_input)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    prover
        .prove(env, DEMO_CIRCUIT_ELF)
        .expect("Failed to generate proof!")
}

pub fn verify(receipt: risc0_zkvm::Receipt) -> bool{
    // let _output: TrieRoot<NodeHash> = receipt.journal.decode().unwrap();
    match receipt.verify(DEMO_CIRCUIT_ID){
        Ok(_) => true,
        Err(_) => false
    }
}

#[test]
fn prove_empty_batch(){
    let db: Rc<MemoryDb<Account>> = Rc::new(MemoryDb::<Account>::empty());
    let old_root_hash: TrieRoot<NodeHash> = TrieRoot::default();
    let operations: Vec<Signed<Transaction>> = vec![];

    let proof: risc0_zkvm::Receipt = prove(db, old_root_hash, operations);
    let is_valid = verify(proof);
    assert_eq!(is_valid, true);
}
