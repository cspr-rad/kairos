{ inputs, ... }:
{
  perSystem = { self', inputs', pkgs, lib, ... }:
    let
      rustToolchain = inputs'.fenix.packages.latest.toolchain;
      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

      kairosProverAttrs = rec {
        pname = "kairos-prover";
        src = lib.fileset.toSource {
          root = ../.;
          fileset = lib.fileset.unions [
            ../kairos-prover
            ../kairos-tx
            ../testdata
          ];
        };
        cargoToml = ./Cargo.toml;
        cargoLock = ./Cargo.lock;
        sourceRoot = "source/kairos-prover";
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        buildInputs = with pkgs; [
          openssl.dev
        ] ++ lib.optionals stdenv.isDarwin [
          libiconv
          darwin.apple_sdk.frameworks.SystemConfiguration
          darwin.apple_sdk.frameworks.Metal
        ];
        cargoVendorDir = craneLib.vendorMultipleCargoDeps {
          inherit (craneLib.findCargoFiles src) cargoConfigs;
          cargoLockList = [
            ./Cargo.lock
          ];
        };

        IGNORE_WRONG_RISC0_IMAGE_ID = "1";
        RISC0_R0VM_PATH = lib.getExe pkgs.r0vm;

        preCheck = ''
          # Proving in CI is disabled because it takes too long.
          # Proving is a test of risc0, not kairos anyway.
          export RISC0_DEV_MODE=1;
        '';

        # Proving in CI is disabled because it takes too long.
        # Proving is a test of risc0, not kairos anyway.
        preBuild = ''
          # The vendored cargo sources will be placed into .cargo-home,
          # however it seems that since the risc0_build crate
          # calls cargo at build time in this directory cargo will be
          # looking for .cargo
          mkdir .cargo
          mv .cargo-home/config.toml .cargo/config.toml
          export RISC0_RUST_SRC=${rustToolchain}/lib/rustlib/src/rust;
        '';
        checkInputs = [ pkgs.r0vm ];
      };
    in
    {
      devShells.risczero = pkgs.mkShell {
        RISC0_RUST_SRC = "${rustToolchain}/lib/rustlib/src/rust";
        RISC0_DEV_MODE = 0;
        RISC0_R0VM_PATH = lib.getExe pkgs.r0vm;
        inputsFrom = [ self'.packages.kairos-prover ];
      };
      packages = {
        kairos-prover-deps = craneLib.buildDepsOnly kairosProverAttrs;

        kairos-prover = craneLib.buildPackage (kairosProverAttrs // {
          cargoArtifacts = self'.packages.kairos-prover-deps;
          meta.mainProgram = "kairos-prover-risc0-server";
        });
      };
    };
  flake = { };
}
