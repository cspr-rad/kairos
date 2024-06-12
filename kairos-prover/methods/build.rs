fn main() {
    let guests = risc0_build::embed_methods();
    match guests.as_slice() {
        [guest] => {
            let hardcoded_id = kairos_verifier_risc0_lib::BATCH_CIRCUIT_PROGRAM_HASH;

            assert_eq!(
                hardcoded_id,
                guest.image_id,
                "`kairos_verifier_risc0_lib::BATCH_CIRCUIT_PROGRAM_HASH = {hardcoded_id:?}` is out of date.\n\
                 Update the hardcoded value to the new hash: {0:?}", guest.image_id
            );
        }
        _ => panic!("Expected exactly one guest method"),
    }
}
