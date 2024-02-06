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
          rustToolchain = inputs'.fenix.packages.stable.toolchain;
          craneLib = inputs.crane.lib.${system}.overrideToolchain rustToolchain;

          kairosNodeAttrs = {
            src = lib.cleanSourceWith {
              src = craneLib.path ./.;
              filter = path: type: craneLib.filterCargoSources path type;
            };
            nativeBuildInputs = with pkgs; [ pkg-config ];

            buildInputs = with pkgs; [
              openssl.dev
            ] ++ lib.optionals stdenv.isDarwin [
              libiconv
            ];
            meta.mainProgram = "kairos-server";
          };
        in
        {
          devShells.default = pkgs.mkShell {
            # Rust Analyzer needs to be able to find the path to default crate
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

            nativeBuildInputs = [
              rustToolchain
            ] ++ kairosNodeAttrs.nativeBuildInputs;
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
          };

          treefmt = {
            projectRootFile = ".git/config";
            programs.nixpkgs-fmt.enable = true;
            programs.rustfmt.enable = true;
            programs.rustfmt.package = craneLib.rustfmt;
            settings.formatter = { };
          };
        };
    };
}
