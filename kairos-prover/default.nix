{ self, inputs, ... }:
{
  perSystem = { config, self', inputs', system, pkgs, ... }:
    {
      devShells.risczero = pkgs.mkShell {
        RISC0_DEV_MODE = 1;
        inputsFrom = [ self'.packages.kairos-prover ];
        nativeBuildInputs = [
          inputs'.risc0pkgs.packages.r0vm
        ];
      };
      packages = {
        kairos-prover = inputs.risc0pkgs.lib.${system}.buildRisc0Package {
          pname = "kairos-prover";
          version = "0.0.1";
          src = ./.;
          doCheck = false;
          cargoSha256 = "sha256-p4arX0k6Onem521suiyz9er2Gvabtk4ANUCrIn7Jd2Y=";
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
