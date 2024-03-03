# Minimum Viable Product for Kairos - A native Casper Transaction Zk Rollup System
This repo contains a complete Kairos demo with a mock L2 state that is used to rollup a limited amount of Transactions for demo purposes.

MVP is a fully functional ZK rollup implementation that utilizes an implementation of Torandocash's Merkle Tree and Risc0 as a proving backend.

In order to accelerate development processes, this version is built in pure Rust and the storage component is effectively a serialized struct.

Production Kairos should support a long transaction history and high througput, but for the scope of a POC / MVP it is advisable to focus on the core components of the rollup, which is exactly what this repo is about.