# Minimum Viable Product for Kairos - A native Casper Transaction Zk Rollup System
This repo contains a complete Kairos demo that is used to rollup a limited amount of Transactions for demo purposes.

Kairos-lab is a private research branch maintained by @jonas089 with @Rom3dius as a collaborator. The goal is to quickly demo the Kairos rollup system and test new state implementations, trees, proving backends, contracts and L1 target architecture. 

MVP is a fully functional ZK rollup implementation that utilizes an implementation of a Delta Merkle Tree and Risc0 as a proving backend.

In order to accelerate development processes, this version is built in pure Rust and the storage component is effectively a serialized struct.

Production Kairos should support a long transaction history and high througput, but for the scope of a POC / MVP it is advisable to focus on the core components of the rollup, which is exactly what this repo is about.


### Testing
In order to test, make sure you have [cargo-nextest](https://nexte.st) and [docker-compose](https://docs.docker.com/compose/install/#scenario-two-install-the-compose-plugin) installed.
You might also need the [jq](https://jqlang.github.io/jq/) cli tool. It comes preinstalled on most linux distros.
Executing `cargo nextest run` will automatically spawn a network using CCTL and a postgresql database.
The environment will stay running after test execution ends until explicitly stopped using the command `docker-compose down` or `docker compose down`. The reasoning behind this is to keep the time waiting on the images to spin up to a minimum while developing and testing the code.