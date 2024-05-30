{ lib, config, ... }:
let
  inherit (lib)
    types
    mkOption
    mkIf
    mkMerge
    mkEnableOption
    ;
  cfg = config.services.cctl;
in
{
  options.services.cctl = {

    enable = mkEnableOption "cctl";

    package = mkOption {
      type = types.package;
    };

    port = mkOption {
      type = types.port;
      default = 11101;
      example = 60000;
      description = ''
        Port to listen on.
        TODO make port configurable in cctl
      '';
    };

    workingDirectory = mkOption {
      type = types.str;
      default = "/var/lib/cctl";
      description = ''
        The working directory path where cctl will put its assets and resources.
      '';
    };

    logLevel = mkOption {
      type = types.str;
      default = "info";
      description = ''
        The log-level that should be used.
      '';
    };
  };

  config = mkIf cfg.enable {

    systemd.services.cctl =
      {
        description = "cctl";
        documentation = [ "" ];
        wantedBy = [ "multi-user.target" ];
        after = [ "network-online.target" ];
        requires = [ "network-online.target" ];
        environment = {
          RUST_LOG = cfg.logLevel;
        };
        serviceConfig =
          mkMerge [
            {
              ExecStart = ''${lib.getExe cfg.package} --working-dir ${cfg.workingDirectory}'';
              Type = "notify";
              Restart = "always";
              User = "cctl";
              Group = "cctl";
              StateDirectory = builtins.baseNameOf cfg.workingDirectory;
              WorkingDirectory = cfg.workingDirectory;
              ReadWritePaths = [
                cfg.workingDirectory
              ];
            }
          ];
      };

    users.users.cctl = {
      name = "cctl";
      group = "cctl";
      isSystemUser = true;
    };
    users.groups.cctl = { };

    # Allows nginx to have read access to the working directory of cctl
    users.users.${config.services.nginx.user}.extraGroups = [ "cctl" ];

    # Since cctl is usually ran on the same machine as the application that is subject to be tested,
    # we need to serve the generated users directory to make it available for client machines
    # when testing
    services.nginx = {
      enable = true;
      virtualHosts."${config.networking.hostName}".locations."/cctl/users/" = {
        alias = "${cfg.workingDirectory}/assets/users/";
        extraConfig = ''
          autoindex on;
          add_header Content-Type 'text/plain charset=UTF-8';
        '';
      };
    };
  };
}
