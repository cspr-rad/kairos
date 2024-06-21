use kairos_circuit_logic::{
    transactions::{L1Deposit, Signed, Withdraw},
    ProofOutputs,
};
use rkyv::Deserialize;
use wasm_bindgen_test::*;

// #[wasm_bindgen_test]
fn rkyv_deserialize() {
    let serialized = &[
        97, 108, 105, 99, 101, 95, 112, 117, 98, 108, 105, 99, 95, 107, 101, 121, 240, 255, 255,
        255, 16, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 97, 108, 105, 99, 101, 95, 112, 117, 98, 108,
        105, 99, 95, 107, 101, 121, 240, 255, 255, 255, 16, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 5, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 177, 191, 11, 99, 82, 80, 117, 150, 40, 153, 179, 9, 170, 82,
        31, 187, 13, 237, 233, 233, 253, 246, 227, 233, 167, 242, 87, 7, 125, 188, 61, 219, 0, 0,
        132, 255, 255, 255, 1, 0, 0, 0, 156, 255, 255, 255, 1, 0, 0, 0,
    ];

    let proof_outputs = rkyv::check_archived_root::<ProofOutputs>(serialized)
        .expect("failed to check archived root");

    let proof_outputs: ProofOutputs = proof_outputs
        .deserialize(&mut rkyv::Infallible)
        .expect("failed to deserialize");

    let expected = ProofOutputs {
        pre_batch_trie_root: None,
        post_batch_trie_root: Some([
            177, 191, 11, 99, 82, 80, 117, 150, 40, 153, 179, 9, 170, 82, 31, 187, 13, 237, 233,
            233, 253, 246, 227, 233, 167, 242, 87, 7, 125, 188, 61, 219,
        ]),
        deposits: Box::new([L1Deposit {
            recipient: vec![
                97, 108, 105, 99, 101, 95, 112, 117, 98, 108, 105, 99, 95, 107, 101, 121,
            ],
            amount: 10,
        }]),
        withdrawals: Box::new([Signed {
            public_key: vec![
                97, 108, 105, 99, 101, 95, 112, 117, 98, 108, 105, 99, 95, 107, 101, 121,
            ],
            nonce: 1,
            transaction: Withdraw { amount: 5 },
        }]),
    };

    assert_eq!(proof_outputs, expected);
}
