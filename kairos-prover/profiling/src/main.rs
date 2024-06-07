use kairos_circuit_logic::ProofInputs;
use methods::PROVE_BATCH_ELF;

use risc0_zkvm::ExecutorEnv;

fn main() {
    let proof_inputs: ProofInputs = serde_json::from_str(include_str!("proof_inputs.json"))
        .expect("Failed to parse example_proof_input.json");

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let env = ExecutorEnv::builder()
        .write(&proof_inputs)
        .map_err(|e| format!("Error in ExecutorEnv builder write: {e}"))
        .unwrap()
        .build()
        .map_err(|e| format!("Error in ExecutorEnv builder build: {e}"))
        .unwrap();

    risc0_zkvm::default_executor()
        .execute(env, PROVE_BATCH_ELF)
        .unwrap();
}
