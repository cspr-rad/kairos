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
        cargoExtraArgs = "--features cuda";
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
        DEP_SPPARK_ROOT = pkgs.fetchFromGitHub {
          owner = "supranational";
          repo = "sppark";
          rev = "v0.1.6";
          sha256 = "sha256-9WyRma3qGhxcPvyoJZm0WwwRmvuFXvotiBnAih4LYNs=";
        };
        RISC0_RUST_SRC = "${rustToolchain}/lib/rustlib/src/rust";
        RISC0_DEV_MODE = 0;
        RISC0_R0VM_PATH = lib.getExe pkgs.r0vm;
        CUDA_PATH = pkgs.cudaPackages.cudatoolkit;
        CUDA_LIBRARY_PATH = let p = pkgs.runCommand "hack" {} ''
          mkdir -p $out/lib
          ln -s ${pkgs.cudaPackages.cudatoolkit}/lib $out/lib/lib64
          ln -s ${pkgs.cudaPackages.cudatoolkit}/include $out/include
        '';
        in "${p}/include:${p}/lib";
        CUDA_ROOT = pkgs.cudaPackages.cudatoolkit;
        CUDA_TOOLKIT_ROOT_DIR = pkgs.cudaPackages.cudatoolkit;
        inputsFrom = [ self.packages.${system}.kairos-prover ];
        shellHook = ''
          export PATH="${pkgs.cudaPackages.cudatoolkit}/nvvm/bin:${pkgs.cudaPackages.cudatoolkit}/lib:$PATH"
          export EXTRA_LDFLAGS="-lcuda"
        '';
        packages = [pkgs.cudaPackages.cudatoolkit pkgs.gcc12]; # for nvcc

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
