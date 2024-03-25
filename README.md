# Kairos

## Development

To get started developing on this repository you can leverage Nix. The following sections explain how to obtain a development shell and how to build the project.

### Development Shell

You have two options here, you can configure and leverage [direnv](https://direnv.net/) to automatically drop you in a development shell (recommended) or do it manually.

#### Automatic Development Shell using `direnv`

First, you will have to install direnv, by adding it to your Nix/NixOS configuration or using your package manager.
Afterward, run:

```sh
touch .envrc
echo "use flake" >> .envrc
direnv allow
```

#### Manual Development Shell

Run:

```sh
nix develop
```

#### Risc0 Development Shell

Because `risc0` projects require a fork of `rustc`, we decided to detach the development shell for `kairos-prover` from the other crates. To enter a development shell when working with the `kairos-prover`/`risc0` projects, run:

If you don't use `direnv`, you will first need to enter the default development shell, if you didn't before:

```sh
nix develop 
```

Afterward (the only command you need when you use `direnv`):

```
nix develop .#risczero
```

### Inside a Development Shell

Inside the development shell, you can use `cargo` as usual during development.

### Formatting

Code for the whole project tree can be formatted by running `nix fmt` from the project's root or anywhere in the tree, but be warned that it will only format code inside the sub-tree.

The `nix fmt` command currently formats all the `Rust` and `Nix` code in the tree. To add support for more languages you'll have to adjust the `treefmt` attribute-set in the `flake.nix` accordingly. A list of already supported formatters can be found [here](https://numtide.github.io/treefmt/formatters/). Note that any formatting tool can be added trivially, if stuck contact your Nix expert.

### Build

You can explore the buildable outputs of this project easily by running:

```sh
nix flake show
```

To build e.g. `kairos` you can then run:

```sh
nix build .#kairos
```

### Check

To run all the "checks" of this project, like formatting, lint, audit, etc. checks, run:

```sh
nix flake check
```

To run a single check e.g. the format check, run:

```sh
nix build .#checks.<system>.treefmt
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
