#rm -rf /Users/chef/.cargo/git/checkouts/casper-node-*
rm -rf /Users/chef/.cargo/git/checkouts/casper-node-6f1becc37e1a8f50/0bcc25f/target/wasm32-unknown-unknown/release
rustup target add wasm32-unknown-unknown

mkdir -p /Users/chef/.cargo/git/checkouts/casper-node-6f1becc37e1a8f50/0bcc25f/target/wasm32-unknown-unknown/release
cargo build -p contract --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ../target/wasm32-unknown-unknown/release/deposit-contract.wasm -o ../target/wasm32-unknown-unknown/release/deposit-contract-optimized.wasm

cargo build -p deposit-session --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ../target/wasm32-unknown-unknown/release/deposit-session.wasm -o ../target/wasm32-unknown-unknown/release/deposit-session-optimized.wasm
cp ../target/wasm32-unknown-unknown/release/deposit-session-optimized.wasm /Users/chef/.cargo/git/checkouts/casper-node-6f1becc37e1a8f50/0bcc25f/target/wasm32-unknown-unknown/release/deposit-session-optimized.wasm
cp ../target/wasm32-unknown-unknown/release/deposit-contract-optimized.wasm /Users/chef/.cargo/git/checkouts/casper-node-6f1becc37e1a8f50/0bcc25f/target/wasm32-unknown-unknown/release/deposit-contract-optimized.wasm

cargo build -p malicious-session --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ../target/wasm32-unknown-unknown/release/malicious-session.wasm -o ../target/wasm32-unknown-unknown/release/malicious-session-optimized.wasm
cp ../target/wasm32-unknown-unknown/release/malicious-session-optimized.wasm /Users/chef/.cargo/git/checkouts/casper-node-6f1becc37e1a8f50/0bcc25f/target/wasm32-unknown-unknown/release/malicious-session-optimized.wasm

cargo build -p withdrawal-session --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ../target/wasm32-unknown-unknown/release/withdrawal-session.wasm -o ../target/wasm32-unknown-unknown/release/withdrawal-session-optimized.wasm
cp ../target/wasm32-unknown-unknown/release/withdrawal-session-optimized.wasm /Users/chef/.cargo/git/checkouts/casper-node-6f1becc37e1a8f50/0bcc25f/target/wasm32-unknown-unknown/release/withdrawal-session-optimized.wasm

cargo build -p malicious-reader --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ../target/wasm32-unknown-unknown/release/malicious-reader.wasm -o ../target/wasm32-unknown-unknown/release/malicious-reader-optimized.wasm
cp ../target/wasm32-unknown-unknown/release/malicious-reader-optimized.wasm /Users/chef/.cargo/git/checkouts/casper-node-6f1becc37e1a8f50/0bcc25f/target/wasm32-unknown-unknown/release/malicious-reader-optimized.wasm
