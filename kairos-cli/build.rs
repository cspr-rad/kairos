use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Path
    let session_binaries_dir = get_wasm_directory();

    // Get the output directory set by Cargo.
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let source_path = Path::new(&session_binaries_dir).join("deposit-session-optimized.wasm");
    let dest_path = Path::new(&out_dir).join("deposit-session-optimized.wasm");

    // Copy the file from the source to the destination
    fs::copy(&source_path, dest_path).expect("Failed to copy WASM file");

    // Print out a message to re-run this script if the source file changes.
    println!("cargo:rerun-if-changed={}", session_binaries_dir.display());
}

pub fn get_wasm_directory() -> PathBuf {
    // Environment variable, or default path based on the project structure.
    let base_path_session = if let Ok(custom_path) = env::var("PATH_TO_SESSION_BINARIES") {
        PathBuf::from(custom_path)
    } else {
        let project_root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(project_root)
            .join("../kairos-session-code/target/wasm32-unknown-unknown/release/")
    };

    if !base_path_session.exists() {
        panic!("WASM directory does not exist: {}. Please build session code at `./kairos-session-code` with `cargo build --release`; or set `PATH_TO_SESSION_BINARIES` env variable.", base_path_session.display());
    }

    // Ensure all WASM files are optimized.
    optimize_files(&base_path_session).expect("Unable to optimize WASM files");

    base_path_session
}

fn optimize_files(dir: &Path) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;

    let mut found_wasm = false;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
            found_wasm = true;

            // Skip already optimized files.
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name.ends_with("-optimized.wasm") {
                continue;
            }

            // Skip if optimized file already exists.
            let optimized_file_name = format!(
                "{}-optimized.wasm",
                file_name.strip_suffix(".wasm").unwrap()
            );
            let optimized_file_path = dir.join(&optimized_file_name);
            if optimized_file_path.exists() {
                continue;
            }

            // Optimize and save as new file.
            let infile = path.to_str().unwrap().to_string();
            let outfile = optimized_file_path.to_str().unwrap().to_string();

            let mut opts = wasm_opt::OptimizationOptions::new_optimize_for_size();
            opts.add_pass(wasm_opt::Pass::StripDebug);
            opts.add_pass(wasm_opt::Pass::SignextLowering);

            opts.run(&infile, &outfile).map_err(|e| e.to_string())?;
        }
    }

    if !found_wasm {
        return Err("No WASM files found in the directory. You should change directory to `./kairos-session-code` and build with `cargo build --release`.".to_string());
    }

    Ok(())
}
