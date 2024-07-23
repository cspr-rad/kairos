use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Path
    let session_binaries_dir = env::var("PATH_TO_SESSION_BINARIES")
        .expect("PATH_TO_SESSION_BINARIES environment variable is not set");

    // Get the output directory set by Cargo.
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let source_path = Path::new(&session_binaries_dir).join("deposit-session-optimized.wasm");
    let dest_path = Path::new(&out_dir).join("deposit-session-optimized.wasm");

    // Copy the file from the source to the destination
    fs::copy(&source_path, dest_path).expect("Failed to copy WASM file");

    // Print out a message to re-run this script if the source file changes.
    println!("cargo:rerun-if-changed={}", source_path.display());
}
