{ config, pkgs, ... }:

let
  marijan = {
    keys = [
      "sk-ssh-ed25519@openssh.com AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAIKJMTVrY0qbJfu2g1TocLxRrYc/AjlnUuR35Y4biaLThAAAABHNzaDo="
    ];
  };
in
{
  nix.settings.trusted-users = [ "deploy" ];
  security.sudo.extraRules = [{
    users = [ "deploy" ];
    commands = [{
      command = "ALL";
      options = [ "NOPASSWD" ];
    }];
  }];
  users = {
    mutableUsers = false;
    users = {
      deploy = {
        isNormalUser = true;
        openssh.authorizedKeys.keys = marijan.keys;
      };
      marijan = {
        isNormalUser = true;
        openssh.authorizedKeys.keys = marijan.keys;
        extraGroups = [ "wheel" ];
        hashedPassword = "$6$Ced3V3xO3wW0kTES$QiZGwCOKI.w9Q4QGTnAnGTIdgMo7DiBcOOs9hOHfGB1R0p75AibbfvCH2ejnHK4qg8rihyF8HoyiiLrKdvgNh/";
      };
    };
  };
}

