### See the risc0 profiling [documentation](https://github.com/risc0/risc0/tree/release-0.21/examples/profiling)

### Viewing the profile

To view the committed profile run `go tool pprof -http=127.0.0.1:8000 profile.pb`
If you don't have `go` and `graphviz` installed, you can run `nix-shell -p pkgs.go pkgs.graphviz` to enter a temporary shell with go installed.

### Generating the profile

`RISC0_PPROF_OUT=./profile.pb cargo run --release`
