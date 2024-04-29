{ self, inputs, ... }:
{
  perSystem = { config, self', inputs', system, pkgs, ... }:
    {
      devShells.risczero = pkgs.mkShell {
        RISC0_RUST_SRC = "${self'.packages.kairos-prover.toolchain}/lib/rustlib/src/rust";
        RISC0_DEV_MODE = 1;
        inputsFrom = [ self.packages.${system}.kairos-prover ];
        nativeBuildInputs = [ inputs'.risc0pkgs.packages.r0vm ];
      };
      packages = {
        kairos-prover = inputs.risc0pkgs.lib.${system}.buildRisc0Package {
          pname = "kairos-prover";
          version = "0.0.1";
          src = ./.;
          cargoHash = "sha256-9II2+wSPaHeKn+sXe3T6f8a3Nhl9ec9wHB9oQC8rHRA=";
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postInstall = ''
            wrapProgram $out/bin/host \
              --set PATH ${pkgs.lib.makeBinPath [ inputs'.risc0pkgs.packages.r0vm ]}
          '';
        };
      };
    };
  flake = { };
}
