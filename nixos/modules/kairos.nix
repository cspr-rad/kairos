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
  cfg = config.services.kairos;
in
{
  options.services.kairos = {

    enable = mkEnableOption (mdDoc "kairos");

    package = mkOption {
      type = types.package;
    };

    port = mkOption {
      type = types.port;
      default = 60000;
      example = 60000;
      description = mdDoc ''
        Port to listen on.
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

    systemd.services.kairos =
      {
        description = "kairos";
        documentation = [ "" ];
        wantedBy = [ "multi-user.target" ];
        after = [ "network-online.target" ];
        requires = [ "network-online.target" ];
        environment = {
          RUST_LOG = cfg.logLevel;
          KAIROS_SERVER_PORT = builtins.toString cfg.port;
        };
        serviceConfig = mkMerge [
          {
            ExecStart = ''${lib.getExe cfg.package}'';
            Restart = "always";
            DynamicUser = true;
          }
        ];
      };
  };
}
