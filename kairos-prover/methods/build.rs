fn main() {
    let guests = risc0_build::embed_methods();

    let ignore_image_id = option_env!("IGNORE_WRONG_RISC0_IMAGE_ID").unwrap_or("1");

    match guests.as_slice() {
        [guest] => {
            let hardcoded_id = kairos_verifier_risc0_lib::BATCH_CIRCUIT_PROGRAM_HASH;

            if matches!(ignore_image_id, "1" | "true" | "True" | "TRUE")
                && hardcoded_id != guest.image_id
            {
                println!(
                    "cargo:warning=Ignoring error: kairos_verifier_risc0_lib::BATCH_CIRCUIT_PROGRAM_HASH = {hardcoded_id:?}`,\
                    but the circuit's new hash is: {0:?}", guest.image_id
                );
            } else {
                assert_eq!(
                hardcoded_id,
                guest.image_id,
                "`kairos_verifier_risc0_lib::BATCH_CIRCUIT_PROGRAM_HASH = {hardcoded_id:?}` is out of date.\n\
                 Update the hardcoded value to the new hash: {0:?}", guest.image_id
            );
            }
        }
        // FIXME the image_id should be stable in devshell and nix
        // Ignore the wrong image_id in nix
        _ => panic!("Expected exactly one guest method"),
    }
}
