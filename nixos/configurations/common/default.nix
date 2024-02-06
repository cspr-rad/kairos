{ pkgs, config, lib, ... }:
{
  imports = [
    ./users.nix
    ./acme.nix
  ];

  nix = {
    settings.trusted-users = [ "@wheel" "root" ];
    package = pkgs.nixFlakes;
    extraOptions =
      ''
        experimental-features = nix-command flakes
      '';
  };

  services.openssh = {
    enable = true;
    settings.PasswordAuthentication = false;
  };
}
