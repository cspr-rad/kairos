{ lib, pkgs, config, ... }:
let
  inherit (lib)
    types
    mkOption
    mkIf
    mkMerge
    mkEnableOption
    mdDoc
    ;
  cfg = config.services.cctl;
in
{
  options.services.cctl = {

    enable = mkEnableOption (mdDoc "cctl");

    package = mkOption {
      type = types.package;
    };

    port = mkOption {
      type = types.port;
      default = 11101;
      example = 60000;
      description = mdDoc ''
        Port to listen on.
        TODO make port configurable in cctl
      '';
    };

    logLevel = mkOption {
      type = types.enum [
        "error"
        "warn"
        "info"
        "debug"
        "trace"
      ];
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
        serviceConfig = mkMerge [
          {
            ExecStart = ''${lib.getExe cfg.package}'';
            Type = "notify";
            Restart = "always";
            DynamicUser = true;
          }
        ];
      };
  };
}
