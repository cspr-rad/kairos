rustup target add wasm32-unknown-unknown

cargo build -p contract --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ./target/wasm32-unknown-unknown/release/deposit-contract.wasm -o ./target/wasm32-unknown-unknown/release/deposit-contract-optimized.wasm

cargo build -p deposit-session --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ./target/wasm32-unknown-unknown/release/deposit-session.wasm -o ./target/wasm32-unknown-unknown/release/deposit-session-optimized.wasm

cargo build -p malicious-session --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ./target/wasm32-unknown-unknown/release/malicious-session.wasm -o ./target/wasm32-unknown-unknown/release/malicious-session-optimized.wasm

cargo build -p withdrawal-session --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ./target/wasm32-unknown-unknown/release/withdrawal-session.wasm -o ./target/wasm32-unknown-unknown/release/withdrawal-session-optimized.wasm

cargo build -p malicious-reader --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ./target/wasm32-unknown-unknown/release/malicious-reader.wasm -o ./target/wasm32-unknown-unknown/release/malicious-reader-optimized.wasm
