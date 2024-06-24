use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=prove_batch_bin");

    let Ok(image_id) = risc0_binfmt::compute_image_id(include_bytes!("prove_batch_bin")) else {
        panic!("Failed to compute image_id");
    };
    let image_id: [u32; 8] = image_id.into();

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let constants = format!(
        "pub const PROVE_BATCH_ID: [u32; 8] = {image_id:?};\n\
         pub const PROVE_BATCH_ELF: &[u8] = include_bytes!(\"{manifest_dir}/prove_batch_bin\");",
    );

    let methods_path = PathBuf::from(&env::var_os("OUT_DIR").unwrap()).join("methods.rs");
    File::create(methods_path)
        .unwrap()
        .write_all(constants.as_bytes())
        .unwrap();

    if kairos_verifier_risc0_lib::BATCH_CIRCUIT_PROGRAM_HASH != image_id {
        panic!(
            "`kairos_verifier_risc0_lib::BATCH_CIRCUIT_PROGRAM_HASH` is out of date\n\
              The new hash is: {image_id:?}\n"
        );
    }
}
