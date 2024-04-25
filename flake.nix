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
    csprpkgs.url = "github:cspr-rad/csprpkgs/add-cctl";
    csprpkgs.inputs.nixpkgs.follows = "nixpkgs";
    hercules-ci-effects.url = "github:hercules-ci/hercules-ci-effects";
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
              filter = path: type:
                (builtins.any (includePath: lib.hasInfix includePath path) [
                  "/kairos-cli"
                  "/kairos-crypto"
                  "/kairos-server"
                  "/kairos-test-utils"
                  "/kairos-tx"
                  "/Cargo.toml"
                  "/Cargo.lock"
                ]) && (
                  # Allow static files.
                  (lib.hasInfix "/tests/fixtures/" path) ||
                  # Default filter (from crane) for .rs files.
                  (craneLib.filterCargoSources path type)
                )
              ;
            };
            nativeBuildInputs = with pkgs; [ pkg-config ];

            buildInputs = with pkgs; [
              openssl.dev
            ] ++ lib.optionals stdenv.isDarwin [
              libiconv
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
            ];
            checkInputs = [
              inputs'.csprpkgs.packages.cctl
            ];
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

            kairos-tx-no-std = craneLib.buildPackage (kairosNodeAttrs // {
              cargoArtifacts = self'.packages.kairos-deps;
              cargoExtraArgs = "-p kairos-tx --no-default-features";
            });

            cctld = pkgs.runCommand "cctld-wrapped"
              {
                buildInputs = [ pkgs.makeWrapper ];
                meta.mainProgram = "cctld";
              }
              ''
                mkdir -p $out/bin
                makeWrapper ${self'.packages.kairos}/bin/cctld $out/bin/cctld \
                  --set PATH ${pkgs.lib.makeBinPath [inputs'.csprpkgs.packages.cctl ]}
              '';

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
              # Default values from https://crane.dev/API.html?highlight=tarpau#cranelibcargotarpaulin
              # --avoid-cfg-tarpaulin fixes nom/bitvec issue https://github.com/xd009642/tarpaulin/issues/756#issuecomment-838769320
              cargoTarpaulinExtraArgs = "--skip-clean --out xml --output-dir $out --avoid-cfg-tarpaulin";
              # For some reason cargoTarpaulin runs the tests in the buildPhase
              buildInputs = kairosNodeAttrs.buildInputs ++ [
                inputs'.csprpkgs.packages.cctl
              ];
            });

            audit = craneLib.cargoAudit {
              inherit (kairosNodeAttrs) src;
              advisory-db = inputs.advisory-db;
              # Default values from https://crane.dev/API.html?highlight=cargoAudit#cranelibcargoaudit
              # FIXME --ignore RUSTSEC-2022-0093 ignores ed25519-dalek 1.0.1 vulnerability caused by introducing casper-client 2.0.0
              # FIXME --ignore RUSTSEC-2024-0013 ignores libgit2-sys 0.14.2+1.5.1 vulnerability caused by introducing casper-client 2.0.0
              cargoAuditExtraArgs = "--ignore yanked --ignore RUSTSEC-2022-0093 --ignore RUSTSEC-2024-0013";
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
      flake =
        {
          herculesCI.ciSystems = [ "x86_64-linux" ];
          effects = { forgeType, remoteHttpUrl, branch, ... }:
            let
              pkgs = import inputs.nixpkgs {
                system = "x86_64-linux";
                overlays = [
                  inputs.hercules-ci-effects.overlay
                ];
              };
              commentOnGh =
                args@{ ...
                }: pkgs.effects.modularEffect (args // {

                  imports = [
                    inputs.hercules-ci-effects.modules.effect.git-auth
                    inputs.hercules-ci-effects.modules.effect.git-auth-gh
                  ];
                  git.checkout.tokenSecret = "gh-token";
                  git.checkout = {
                    inherit forgeType;
                    remoteUrl = remoteHttpUrl;
                  };
                  effectScript = ''
                    gh pr comment ${branch} --body-test "test"
                  '';
                });
            in
            {
              testComment = commentOnGh { };
            };
        };
    };
}
