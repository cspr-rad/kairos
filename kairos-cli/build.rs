use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Rerun the build script if the environment variable changes.
    println!("cargo:rerun-if-env-changed=PATH_TO_SESSION_BINARIES");

    // Determine the session binaries directory.
    let (is_automatic_path, session_binaries_dir) = if let Ok(session_code_dir) = env::var("PATH_TO_SESSION_BINARIES") {
        (false, PathBuf::from(session_code_dir))
    } else {
        // Run `cargo build --release` if the environment variable is not set.
        println!("cargo:warning=Building session code dependency.");
        let project_root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(Path::new(&project_root).join("../kairos-session-code"))
            .status()
            .expect("Failed to execute cargo build --release");

        if !status.success() {
            panic!("cargo build --release failed");
        }

        (true, get_default_wasm_directory(&project_root))
    };

    // Rerun the build script if the session binaries directory changes.
    println!("cargo:rerun-if-changed={}", session_binaries_dir.display());

    // Ensure all WASM files are optimized.
    // NOTE: We skip it for Nix (it relies on env variable), as files are already optimized and read-only.
    if is_automatic_path {
        optimize_files(&session_binaries_dir).expect("Unable to optimize WASM files");
    }

    // Get the output directory set by Cargo.
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let source_path = Path::new(&session_binaries_dir).join("deposit-session-optimized.wasm");
    let dest_path = Path::new(&out_dir).join("deposit-session-optimized.wasm");

    // Copy the file from the source to the destination
    fs::copy(source_path, dest_path).expect("Failed to copy WASM file");
}

fn get_default_wasm_directory(project_root: &str) -> PathBuf {
    let base_path_session = PathBuf::from(project_root)
        .join("../kairos-session-code/target/wasm32-unknown-unknown/release/");

    if !base_path_session.exists() {
        panic!("WASM directory does not exist: {}. Please build session code at `./kairos-session-code` with `cargo build --release`; or set `PATH_TO_SESSION_BINARIES` env variable.", base_path_session.display());
    }

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

            // Warn about file that will be overwritten.
            let optimized_file_name = format!(
                "{}-optimized.wasm",
                file_name.strip_suffix(".wasm").unwrap()
            );
            let optimized_file_path = dir.join(&optimized_file_name);
            if optimized_file_path.exists() {
                println!("cargo:warning=Overwriting {}", optimized_file_name);
                //continue; // NOTE: Uncomment to disable overwrite.
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
        return Err("No WASM files found.".to_string());
    }

    Ok(())
}
