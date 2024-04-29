use circuits::{
    DEMO_CIRCUIT_ELF, DEMO_CIRCUIT_ID
};
use risc0_zkvm::{default_prover, ExecutorEnv};
use sha2::Sha256;

use kairos_trie::{
    stored::{memory_db::MemoryDb, merkle::SnapshotBuilder}, NodeHash, TrieRoot,
    stored::merkle::Snapshot, DigestHasher
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

    let env: ExecutorEnv<'_> = ExecutorEnv::builder()
        .write(&circuit_input)
        .unwrap()
        .build()
        .unwrap();

    run_against_active_prover(env)
}

pub fn verify(receipt: risc0_zkvm::Receipt) -> bool{
    // let _output: TrieRoot<NodeHash> = receipt.journal.decode().unwrap();
    match receipt.verify(DEMO_CIRCUIT_ID){
        Ok(_) => true,
        Err(_) => false
    }
}

#[cfg(not(feature="groth16"))]
fn run_against_active_prover(env: ExecutorEnv<'_>) -> risc0_zkvm::Receipt {
    let prover = default_prover();
    prover
        .prove(env, DEMO_CIRCUIT_ELF)
        .expect("Failed to generate proof!")
}

#[cfg(feature = "groth16")]
fn run_against_active_prover(env: ExecutorEnv<'_>) -> risc0_zkvm::Receipt {
    use risc0_groth16::docker::stark_to_snark;
    let mut exec = ExecutorImpl::from_elf(env, DEMO_CIRCUIT_ELF).unwrap();
    let session = exec.run().unwrap();
    let opts = ProverOpts::default();
    let ctx = VerifierContext::default();
    let prover = get_prover_server(&opts).unwrap();
    let receipt = prover.prove_session(&ctx, &session).unwrap();

    let claim = receipt.get_claim().unwrap();
    let composite_receipt = receipt.inner.composite().unwrap();
    let succinct_receipt = prover.compress(composite_receipt).unwrap();
    let journal = session.journal.unwrap().bytes;
    let ident_receipt = identity_p254(&succinct_receipt).unwrap();
    let seal_bytes = ident_receipt.get_seal_bytes();
    let seal = stark_to_snark(&seal_bytes).unwrap().to_vec();
    Receipt::new(
        InnerReceipt::Compact(CompactReceipt { seal, claim }),
        journal,
    )
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
