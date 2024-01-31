{ self, inputs, ... }:
{
  flake = {
    nixosModules = {
      kairos =
        { pkgs, lib, ... }:
        {
          imports = [ ./modules/kairos.nix ];
          services.kairos.package = self.packages.${pkgs.system}.kairos;
        };
    };
  };
}
