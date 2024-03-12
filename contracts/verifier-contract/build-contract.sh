rustup target add wasm32-unknown-unknown

cargo build -p verifier-contract --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ./target/wasm32-unknown-unknown/release/verifier-contract.wasm -o ./target/wasm32-unknown-unknown/release/verifier-contract-optimized.wasm