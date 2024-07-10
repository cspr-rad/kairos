{
  description = "kairos";

  nixConfig = {
    extra-substituters = [
      "https://crane.cachix.org"
      "https://nix-community.cachix.org"
      "https://kairos.cachix.org"
      "https://casper-cache.marijan.pro"
    ];
    extra-trusted-public-keys = [
      "crane.cachix.org-1:8Scfpmn9w+hGdXH/Q9tTLiYAE/2dnJYRJP7kl80GuRk="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "kairos.cachix.org-1:1EqnyWXEbd4Dn1jCbiWOF1NLOc/bELx+wuqk0ZpbeqQ="
      "casper-cache.marijan.pro:XIDjpzFQTEuWbnRu47IqSOy6IqyZlunVGvukNROL850="
    ];
  };

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    advisory-db.url = "github:rustsec/advisory-db";
    advisory-db.flake = false;
    cctl.url = "github:casper-network/cctl/947c34b991e37476db82ccfa2bd7c0312c1a91d7";
    csprpkgs.follows = "cctl/csprpkgs";
  };

  outputs = inputs@{ self, flake-parts, treefmt-nix, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "x86_64-darwin" "aarch64-darwin" ];
      imports = [
        treefmt-nix.flakeModule
        ./kairos-prover
        ./nixos
      ];
      perSystem = { config, self', inputs', system, pkgs, lib, ... }:
        let
          rustToolchain = with inputs'.fenix.packages; combine [
            stable.toolchain
            targets.wasm32-unknown-unknown.stable.rust-std
          ];
          craneLib = inputs.crane.lib.${system}.overrideToolchain rustToolchain;

          # TODO reuse in nixos tests
          cctlConfig = {
            chainspec = pkgs.fetchurl {
              url = "https://raw.githubusercontent.com/cspr-rad/casper-node/53136ac5f004f2ae70a75b4eeb2ff7d907aff6aa/resources/local/chainspec.toml.in";
              hash = "sha256-b/6c5o3JXFlaTgTHxs8JepaHzjMG75knzlKKqRd/7pc=";
            };
            config = pkgs.fetchurl {
              url = "https://raw.githubusercontent.com/cspr-rad/casper-node/53136ac5f004f2ae70a75b4eeb2ff7d907aff6aa/resources/local/config.toml";
              hash = "sha256-ZuNbxw0nBjuONEZRK8Ru96zZQak4MEQ/eM1fA6esyCM=";
            };
          };

          kairosContractsAttrs = {
            src = lib.cleanSourceWith {
              src = lib.fileset.toSource {
                root = ./.;
                fileset = lib.fileset.unions [
                  ./kairos-contracts
                  ./kairos-prover/kairos-circuit-logic
                  ./kairos-prover/kairos-verifier-risc0-lib
                ];
              };
              filter = path: type: craneLib.filterCargoSources path type;
            };
            cargoToml = ./kairos-contracts/Cargo.toml;
            cargoLock = ./kairos-contracts/Cargo.lock;
            sourceRoot = "source/kairos-contracts";

            cargoExtraArgs = "--target wasm32-unknown-unknown";
            CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
            nativeBuildInputs = [ pkgs.binaryen pkgs.lld pkgs.llvmPackages.bintools ];
            doCheck = false;
            # Append "-optimized" to wasm files, to make the tests pass
            postInstall = ''
              directory="$out/bin/"
              for file in "$directory"*.wasm; do
                if [ -e "$file" ]; then
                  # Extract the file name without extension
                  filename=$(basename "$file" .wasm)
                  # Append "-optimized" to the filename and add back the .wasm extension
                  new_filename="$directory$filename-optimized.wasm"
                  wasm-opt -Oz --strip-debug --signext-lowering "$file" -o "$new_filename"
                fi
              done
            '';
          };

          kairosSessionCodeAttrs = {
            src = lib.cleanSourceWith {
              src = lib.fileset.toSource {
                root = ./.;
                fileset = lib.fileset.unions [
                  ./kairos-session-code
                ];
              };
              filter = path: type: craneLib.filterCargoSources path type;
            };
            cargoToml = ./kairos-session-code/Cargo.toml;
            cargoLock = ./kairos-session-code/Cargo.lock;
            sourceRoot = "source/kairos-session-code";

            cargoExtraArgs = "--target wasm32-unknown-unknown";
            CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
            nativeBuildInputs = [ pkgs.binaryen pkgs.lld pkgs.llvmPackages.bintools ];
            doCheck = false;
            # Append "-optimized" to wasm files, to make the tests pass
            postInstall = ''
              directory="$out/bin/"
              for file in "$directory"*.wasm; do
                if [ -e "$file" ]; then
                  # Extract the file name without extension
                  filename=$(basename "$file" .wasm)
                  # Append "-optimized" to the filename and add back the .wasm extension
                  new_filename="$directory$filename-optimized.wasm"
                  wasm-opt -Oz --strip-debug --signext-lowering "$file" -o "$new_filename"
                fi
              done
            '';
          };

          kairosNodeAttrs = {
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./casper-deploy-notifier
                ./demo-contract-tests
                ./kairos-cli
                ./kairos-crypto
                ./kairos-server
                ./kairos-test-utils
                ./kairos-tx
                ./kairos-prover/kairos-circuit-logic
                ./kairos-prover/kairos-verifier-risc0-lib
                ./kairos-contracts/demo-contract/contract-utils
                ./testdata
              ];
            };

            nativeBuildInputs = [ pkgs.binaryen pkgs.lld pkgs.llvmPackages.bintools pkgs.pkg-config ];
            buildInputs = with pkgs; [
              openssl.dev
            ] ++ lib.optionals stdenv.isDarwin [
              libiconv
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
            ];
            checkInputs = [
              inputs'.cctl.packages.cctl
            ];

            CASPER_CHAIN_NAME = "cspr-dev-cctl";
            PATH_TO_WASM_BINARIES = "${self'.packages.kairos-contracts}/bin";
            PATH_TO_SESSION_BINARIES = "${self'.packages.kairos-session-code}/bin";
            CCTL_CONFIG = "${cctlConfig.config}";
            CCTL_CHAINSPEC = "${cctlConfig.chainspec}";

            meta.mainProgram = "kairos-server";
          };
        in
        {
          devShells.default = pkgs.mkShell {
            # Rust Analyzer needs to be able to find the path to default crate
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
            CASPER_CHAIN_NAME = "cspr-dev-cctl";
            PATH_TO_WASM_BINARIES = "${self'.packages.kairos-contracts}/bin";
            PATH_TO_SESSION_BINARIES = "${self'.packages.kairos-session-code}/bin";
            CCTL_CONFIG = "${cctlConfig.config}";
            CCTL_CHAINSPEC = "${cctlConfig.chainspec}";
            inputsFrom = [ self'.packages.kairos self'.packages.kairos-contracts ];
          };

          packages = {
            kairos-deps = craneLib.buildDepsOnly (kairosNodeAttrs // {
              pname = "kairos";
            });

            kairos = craneLib.buildPackage (kairosNodeAttrs // {
              cargoArtifacts = self'.packages.kairos-deps;
            });

            kairos-tx-no-std = craneLib.buildPackage (kairosNodeAttrs // {
              cargoArtifacts = self'.packages.kairos-deps;
              cargoExtraArgs = "-p kairos-tx --no-default-features";
            });

            kairos-crypto-no-std = craneLib.buildPackage (kairosNodeAttrs // {
              cargoArtifacts = self'.packages.kairos-deps;
              cargoExtraArgs = "-p kairos-crypto --no-default-features --features crypto-casper,tx";
            });

            cctld = pkgs.runCommand "cctld-wrapped"
              {
                buildInputs = [ pkgs.makeWrapper ];
                meta.mainProgram = "cctld";
              }
              ''
                mkdir -p $out/bin
                makeWrapper ${self'.packages.kairos}/bin/cctld $out/bin/cctld \
                  --set PATH ${pkgs.lib.makeBinPath [inputs'.cctl.packages.cctl ]}
              '';

            default = self'.packages.kairos;

            kairos-docs = craneLib.cargoDoc (kairosNodeAttrs // {
              cargoArtifacts = self'.packages.kairos-deps;
            });

            kairos-contracts-deps = craneLib.buildPackage (kairosContractsAttrs // {
              pname = "kairos-contracts";
            });

            kairos-contracts = craneLib.buildPackage (kairosContractsAttrs // {
              cargoArtifacts = self'.packages.kairos-contracts-deps;
            });

            kairos-session-code-deps = craneLib.buildPackage (kairosSessionCodeAttrs // {
              pname = "kairos-session-code-deps";
            });

            kairos-session-code = craneLib.buildPackage (kairosSessionCodeAttrs // {
              pname = "kairos-session-code";
            });
          };

          checks = {
            lint = craneLib.cargoClippy (kairosNodeAttrs // {
              cargoArtifacts = self'.packages.kairos-deps;
              cargoClippyExtraArgs = "--features=all-tests --all-targets -- --deny warnings";
            });

            #coverage-report = craneLib.cargoTarpaulin (kairosNodeAttrs // {
            #  cargoArtifacts = self'.packages.kairos-deps;
            #  # FIXME fix weird issue with rust-nightly and tarpaulin https://github.com/xd009642/tarpaulin/issues/1499
            #  RUSTFLAGS = "-Cstrip=none";
            #  # Default values from https://crane.dev/API.html?highlight=tarpau#cranelibcargotarpaulin
            #  # --avoid-cfg-tarpaulin fixes nom/bitvec issue https://github.com/xd009642/tarpaulin/issues/756#issuecomment-838769320
            #  cargoTarpaulinExtraArgs = "--features=all-tests --skip-clean --out xml --output-dir $out --avoid-cfg-tarpaulin";
            #  # For some reason cargoTarpaulin runs the tests in the buildPhase
            #  buildInputs = kairosNodeAttrs.buildInputs ++ [
            #    inputs'.csprpkgs.packages.cctl
            #  ];
            #});

            # See https://github.com/cspr-rad/kairos/security/dependabot for this functionality
            # audit = craneLib.cargoAudit {
            #   inherit (kairosNodeAttrs) src;
            #   advisory-db = inputs.advisory-db;
            #   # Default values from https://crane.dev/API.html?highlight=cargoAudit#cranelibcargoaudit
            #   # FIXME --ignore RUSTSEC-2022-0093 ignores ed25519-dalek 1.0.1 vulnerability caused by introducing casper-client 2.0.0
            #   # FIXME --ignore RUSTSEC-2024-0013 ignores libgit2-sys 0.14.2+1.5.1 vulnerability caused by introducing casper-client 2.0.0
            #   cargoAuditExtraArgs = "--ignore yanked --ignore RUSTSEC-2022-0093 --ignore RUSTSEC-2024-0013";
            # };

            kairos-contracts-lint = craneLib.cargoClippy (kairosContractsAttrs // {
              cargoArtifacts = self'.packages.kairos-contracts-deps;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            });

            kairos-session-code-lint = craneLib.cargoClippy (kairosSessionCodeAttrs // {
              cargoArtifacts = self'.packages.kairos-session-code-deps;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            });

            #   kairos-contracts-audit = craneLib.cargoAudit {
            #     inherit (kairosContractsAttrs) src;
            #     advisory-db = inputs.advisory-db;
            #     # Default values from https://crane.dev/API.html?highlight=cargoAudit#cranelibcargoaudit
            #     # FIXME --ignore RUSTSEC-2022-0093 ignores ed25519-dalek 1.0.1 vulnerability caused by introducing casper-client 2.0.0
            #     # FIXME --ignore RUSTSEC-2022-0054 wee_alloc is Unmaintained caused by introducing casper-contract
            #     cargoAuditExtraArgs = "--ignore yanked --deny warnings --ignore RUSTSEC-2022-0093 --ignore RUSTSEC-2022-0054";
            #   };
          };

          treefmt = {
            projectRootFile = ".git/config";
            programs.nixpkgs-fmt.enable = true;
            programs.rustfmt.enable = true;
            programs.rustfmt.package = craneLib.rustfmt;
            settings.formatter = { };
          };
        };
      flake = {
        herculesCI.ciSystems = [ "x86_64-linux" ];
      };
    };
}
