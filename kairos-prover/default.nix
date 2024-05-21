{ self, inputs, ... }:
{
  perSystem = { config, self', inputs', system, pkgs, lib, ... }:
    {
      devShells.risczero = pkgs.mkShell {
        RISC0_RUST_SRC = "${self'.packages.kairos-prover.toolchain}/lib/rustlib/src/rust";
        RISC0_DEV_MODE = 1;
        inputsFrom = [ self.packages.${system}.kairos-prover ];
        # I cannot install Metal via Nix, so you need to follow the standard xcode metal installation instructions
        buildInputs = lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [ Metal SystemConfiguration ]);
        nativeBuildInputs = [ inputs'.risc0pkgs.packages.r0vm ];
      };
      packages = {
        kairos-prover = inputs.risc0pkgs.lib.${system}.buildRisc0Package {
          pname = "kairos-prover";
          version = "0.0.1";
            src = lib.cleanSourceWith {
              src = ../.;
              filter = path: type:
                (builtins.any (includePath: lib.hasInfix includePath path) [
                  "/kairos-prover"
                  "/kairos-tx"
                  "/kairos-circuit-logic"
                  "/Cargo.toml"
                ]);
              };
          sourceRoot = "source/kairos-prover";
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postInstall = ''
            wrapProgram $out/bin/kairos-prover-risc0-server \
              --set PATH ${pkgs.lib.makeBinPath [ inputs'.risc0pkgs.packages.r0vm ]}
          '';
        };
      };
    };
  flake = { };
}
