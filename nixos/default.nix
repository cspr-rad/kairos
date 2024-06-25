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
  perSystem = { inputs', ... }: {
    packages.casper-client-rs = inputs'.csprpkgs.packages.casper-client-rs.overrideAttrs (_: prev: {
      # the patch can be obtained from https://github.com/cspr-rad/casper-client-rs/pull/2 by appending ".patch" to the URL
      patches = prev.patches ++ [ ../patches/casper-client-deploy-size.patch ];
      # disable checks because of this test https://github.com/casper-ecosystem/casper-client-rs/blob/release-2.0.0/lib/cli/tests.rs#L135
      doCheck = false;
    });
  };
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
              inherit (self.packages.${pkgs.system}) kairos kairos-contracts casper-client-rs;
              cctlModule = self.nixosModules.cctl;
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
