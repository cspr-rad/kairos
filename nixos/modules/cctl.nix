{ lib, pkgs, config, ... }:
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
          let
            stateDirectory = "cctl";
            workingDirectory = "/var/lib/${stateDirectory}";
          in
          mkMerge [
            {
              ExecStart = ''${lib.getExe cfg.package} --working-dir ${workingDirectory}'';
              Type = "notify";
              Restart = "always";
              DynamicUser = true;
              StateDirectory = "cctl";
              WorkingDirectory = workingDirectory;
              ReadWritePaths = [
                workingDirectory
              ];
            }
          ];
      };
  };
}
