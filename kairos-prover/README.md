## Rust Analyzer Config

Make sure `rust-analyzer` is not enabling features `prove`, `metal`, `cuda`, `disable-dev-mode`.
These features are surfaced from `risc0` in this workspace.
If they are enabled in `rust-analyzer` they will prevent `rust-analyzer` from providing diagnostics and start long running `clang++` compilations that lock `./target`.
If cargo is stuck with a `./target` is locked message for an extended period of time run `killall clang++`.

In vscode or neovim settings make sure `rust-analyzer.cargo.features = []` does not include `all` and the legacy option `rust-analyzer.cargo.allFeatures = false`.
