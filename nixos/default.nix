{ self, inputs, ... }:
let
  mkKairosHostConfig = hostName:
    ({ config, ... }:
      {
        imports = [
          self.nixosModules.kairos
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
              hostConfiguration = mkKairosHostConfig "kairos-host";
              verifyServices = [ "kairos.service" ];
            };
        kairos-end-to-end-test =
          pkgs.callPackage
            ./tests/end-to-end.nix
            {
              inherit mkKairosHostConfig;
              inherit (self.packages.${pkgs.system}) kairos;
              cctlModule = self.nixosModules.cctl;
              inherit (inputs.csprpkgs.packages.${pkgs.system}) casper-client-rs;
            };
      };
    nixosModules = {
      kairos =
        { pkgs, lib, ... }:
        {
          imports = [ ./modules/kairos.nix ];
          services.kairos.package = self.packages.${pkgs.system}.kairos;
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
