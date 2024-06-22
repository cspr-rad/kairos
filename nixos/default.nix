{ self, inputs, ... }:
let
  mkKairosHostConfig = hostName:
    ({ config, ... }:
      {
        imports = [
          self.nixosModules.kairos
          self.nixosModules.kairos-prover
          ./configurations/kairos-host
          ({ config, ... }: {
            networking.hostName = hostName;
          })
        ];
      });
in
{
  flake = {
    checks."x86_64-linux" =
      let pkgs = inputs.nixpkgs.legacyPackages."x86_64-linux";
      in
      {
        verify-kairos-host-configuration-test =
          pkgs.callPackage
            ./tests/verify-host-configuration.nix
            {
              hostConfiguration = {
                imports = [
                  (mkKairosHostConfig "kairos-host")
                ];
                # A placeholder URL to make the test pass
                services.kairos.casperRpcUrl = "http://localhost:11101/rpc";
              };
              verifyServices = [ "kairos.service" ];
            };
        kairos-end-to-end-test =
          pkgs.callPackage
            ./tests/end-to-end.nix
            {
              inherit mkKairosHostConfig;
              inherit (self.packages.${pkgs.system}) kairos kairos-contracts;
              cctlModule = self.nixosModules.cctl;
              inherit (inputs.csprpkgs.packages.${pkgs.system}) casper-client-rs casper-node;
            };
      };
    nixosModules = {
      kairos =
        { pkgs, lib, ... }:
        {
          imports = [ ./modules/kairos.nix ];
          services.kairos.package = self.packages.${pkgs.system}.kairos;
        };
      kairos-prover =
        { pkgs, ... }:
        {
          imports = [ ./modules/kairos-prover.nix ];
          services.kairos-prover.package = self.packages.${pkgs.system}.kairos-prover;
        };
      cctl =
        { pkgs, lib, ... }:
        {
          imports = [ ./modules/cctl.nix ];
          services.cctl.package = self.packages.${pkgs.system}.cctld;
        };
    };
  };
}
