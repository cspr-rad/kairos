{ lib, pkgs, config, ... }:
let
  inherit (lib)
    types
    mkOption
    mkIf
    mkMerge
    mkEnableOption
    ;
  cfg = config.services.kairos-prover;
in
{
  options.services.kairos-prover = {

    enable = mkEnableOption "kairos";

    package = mkOption {
      type = types.package;
    };

    bindAddress = mkOption {
      type = types.str;
      default = "0.0.0.0";
      example = "0.0.0.0";
      description = ''
        Port to listen on.
      '';
    };

    port = mkOption {
      type = types.port;
      default = 60001;
      example = 60001;
      description = ''
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

    systemd.services.kairos-prover =
      {
        description = "kairos-prover";
        documentation = [ "" ];
        wantedBy = [ "multi-user.target" ];
        after = [ "network-online.target" ];
        requires = [ "network-online.target" ];
        environment = {
          RUST_LOG = cfg.logLevel;
          KAIROS_PROVER_SERVER_SOCKET_ADDR = "${cfg.bindAddress}:${builtins.toString cfg.port}";
        };
        serviceConfig = mkMerge [
          {
            ExecStart = "${lib.getExe cfg.package}";
            Restart = "always";
            DynamicUser = true;
          }
        ];
      };
  };
}
