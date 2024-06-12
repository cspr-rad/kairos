fn main() {
    let guests = risc0_build::embed_methods();
    match guests.as_slice() {
        [guest] => {
            let program_id_bytes: Vec<u8> = guest
                .image_id
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect();

            let hexed_program_id = hex::encode(program_id_bytes);

            let mut env_id = std::env::var("KAIROS_BATCH_CIRCUIT_PROGRAM_HASH");

            // If the env var is not set (by nix), try to load it from .env
            if env_id.is_err() {
                let _ = dotenvy::dotenv();
                env_id = std::env::var("KAIROS_BATCH_CIRCUIT_PROGRAM_HASH");
            }

            match env_id {
                Ok(env_hash) if env_hash == hexed_program_id => {}
                Ok(env_hash) => {
                    panic!("`KAIROS_BATCH_CIRCUIT_PROGRAM_HASH={env_hash}` is out of date.\n\
                            set `KAIROS_BATCH_CIRCUIT_PROGRAM_HASH=\"{hexed_program_id}\"` in nix and .env");
                }
                Err(e) => {
                    panic!("`KAIROS_BATCH_CIRCUIT_PROGRAM_HASH` is not set: {e}\n\
                            set `KAIROS_BATCH_CIRCUIT_PROGRAM_HASH=\"{hexed_program_id}\"` in nix and .env");
                }
            }
        }
        _ => panic!("Expected exactly one guest method"),
    }
}
