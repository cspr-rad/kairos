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

    bindAddress = mkOption {
      type = types.str;
      default = "0.0.0.0";
      example = "0.0.0.0";
      description = mdDoc ''
        Port to listen on.
      '';
    };

    port = mkOption {
      type = types.port;
      default = 60000;
      example = 60000;
      description = mdDoc ''
        Port to listen on.
      '';
    };

    casperRpcUrl = mkOption {
      type = types.str;
      example = "http://127.0.0.1:11101/rpc";
      description = ''
        A casper node URL.
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
          KAIROS_SERVER_SOCKET_ADDR = "${cfg.bindAddress}:${builtins.toString cfg.port}";
          KAIROS_SERVER_CASPER_RPC = "${cfg.casperRpcUrl}";
          KAIROS_SERVER_CASPER_CONTRACT_HASH = "0000000000000000000000000000000000000000000000000000000000000000";
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
