{
  description = "kairos";

  nixConfig = {
    extra-substituters = [
      "https://crane.cachix.org"
      "https://nix-community.cachix.org"
      "https://kairos.cachix.org"
    ];
    extra-trusted-public-keys = [
      "crane.cachix.org-1:8Scfpmn9w+hGdXH/Q9tTLiYAE/2dnJYRJP7kl80GuRk="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "kairos.cachix.org-1:1EqnyWXEbd4Dn1jCbiWOF1NLOc/bELx+wuqk0ZpbeqQ="
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
    risc0pkgs.url = "github:cspr-rad/risc0pkgs";
    risc0pkgs.inputs.nixpkgs.follows = "nixpkgs";
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
            complete.toolchain
            targets.wasm32-unknown-unknown.latest.rust-std
            targets.aarch64-apple-darwin.complete.rust-std
            targets.x86_64-apple-darwin.complete.rust-std
          ];
          craneLib = inputs.crane.lib.${system}.overrideToolchain rustToolchain;

          kairosContractsAttrs = {
            src = lib.cleanSourceWith {
              src = craneLib.path ./contracts;
              filter = path: type: craneLib.filterCargoSources path type;
            };
            cargoExtraArgs = "--target wasm32-unknown-unknown";
            nativeBuildInputs = [ pkgs.binaryen ];
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
                  wasm-opt --strip-debug --signext-lowering "$file" -o "$new_filename"
                  #mv "$file" "$new_filename"
                fi
              done
            '';
          };

          kairosNodeAttrs = {
            src = lib.cleanSourceWith {
              src = craneLib.path ./.;
              filter = path: type:
                # Allow static files.
                (lib.hasInfix "/fixtures/" path) ||
                # ignore the contracts directory
                !(lib.hasInfix "contracts/" path) ||
                # Default filter (from crane) for .rs files.
                (craneLib.filterCargoSources path type)
              ;
            };
            nativeBuildInputs = with pkgs; [ pkg-config ];

            buildInputs = with pkgs; [
              openssl.dev
            ] ++ lib.optionals stdenv.isDarwin [
              libiconv
            ];

            PATH_TO_WASM_BINARIES = "${self'.packages.kairos-contracts}/bin";

            meta.mainProgram = "kairos-server";
          };
        in
        {
          devShells.default = pkgs.mkShell {
            # Rust Analyzer needs to be able to find the path to default crate
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            inputsFrom = [ self'.packages.kairos ];
          };

          packages = {
            kairos-deps = craneLib.buildDepsOnly (kairosNodeAttrs // {
              pname = "kairos";
            });

            kairos = craneLib.buildPackage (kairosNodeAttrs // {
              cargoArtifacts = self'.packages.kairos-deps;
            });

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
          };

          checks = {
            lint = craneLib.cargoClippy (kairosNodeAttrs // {
              cargoArtifacts = self'.packages.kairos-deps;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            });

            coverage-report = craneLib.cargoTarpaulin (kairosNodeAttrs // {
              cargoArtifacts = self'.packages.kairos-deps;
            });

            audit = craneLib.cargoAudit {
              inherit (kairosNodeAttrs) src;
              advisory-db = inputs.advisory-db;
            };

            kairos-contracts-lint = craneLib.cargoClippy (kairosContractsAttrs // {
              cargoArtifacts = self'.packages.kairos-contracts-deps;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            });

            kairos-contracts-audit = craneLib.cargoAudit {
              inherit (kairosContractsAttrs) src;
              advisory-db = inputs.advisory-db;
            };
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
