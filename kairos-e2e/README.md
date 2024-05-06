# kairos-e2e

## What?
This crate allows for end-to-end testing of a crate. It allows you to register futures as tests that will be run after the testing environment is setup, all within a Rust codebase. You can specify/create any fixture you'd like, then specify which fixture you'd like to use per-crate.

## Why?
This allows you to do setup and teardown of testing infrastructure from Rust instead of requiring external tooling. It also enables the usage of `cargo nextest` which dramatically speeds up testing and includes lots of CI tooling.

## How to use?
In your crates `Cargo.toml` you add the following:
```
[[test]]
name = "project-name.rs"
harness = false
```
Then in the root of your crate, create a folder named `tests` and a child file with the same name you specified in your `Cargo.toml`. Then you can use the following to add and run a test.
```
use kairos_e2e::{register_test, Test, E2EResult};

async fn test_kairos_e2e() -> E2EResult {
    println!("This test was ran with kairos-e2e");
    E2EResult::passed()
}

register_test!("test_kairos_e2e", test_kairos_e2e);

fn main() {
    kairos_e2e::run_tests();
}
```
You can also split up tests into multiple files.