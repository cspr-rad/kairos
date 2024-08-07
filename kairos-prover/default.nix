{ self, inputs, ... }:
{
  perSystem = { self', inputs', system, pkgs, lib, ... }:
    let
      rustToolchain = inputs'.fenix.packages.latest.toolchain;
      craneLib = inputs.crane.lib.${system}.overrideToolchain rustToolchain;

      rustup-mock = pkgs.writeShellApplication {
        name = "rustup";
        text = ''
          # the buildscript uses rustup toolchain to check
          # whether the risc0 toolchain was installed
          if [[ "$1" = "toolchain" ]]
          then
            printf "risc0\n"
          elif [[ "$1" = "+risc0" ]]
          then
            printf "${rustToolchain}/bin/rustc"
          fi
        '';
      };

      kairosProverAttrs = rec {
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
          cargo-risczero
        ];
        buildInputs = with pkgs; [
          openssl.dev
        ] ++ lib.optionals stdenv.isDarwin [
          libiconv
          darwin.apple_sdk.frameworks.SystemConfiguration
          darwin.apple_sdk.frameworks.Metal
        ];
        cargoVendorDir = inputs.crane.lib.${system}.vendorMultipleCargoDeps {
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
        inputsFrom = [ self.packages.${system}.kairos-prover ];
      };
      packages = {
        kairos-prover-deps = craneLib.buildDepsOnly (kairosProverAttrs // {
          pname = "kairos";
        });

        kairos-prover = craneLib.buildPackage (kairosProverAttrs // {
          cargoArtifacts = self'.packages.kairos-prover-deps;
          meta.mainProgram = "kairos-prover-risc0-server";
        });
      };
    };
  flake = { };
}
