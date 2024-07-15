{ lib, pkgs, config, ... }:
let
  inherit (lib)
    types
    mkOption
    mkIf
    mkMerge
    mkEnableOption
    mdDoc
    optionalAttrs
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
        The casper node URL to the RPC endpoint.
      '';
    };

    demoContractHash = mkOption {
      type = types.str;
      example = "TODO put a contract hash here";
      description = ''
        The hash of the deployed demo contract.
        Use an empty string when testing with cctl.
      '';
    };

    casperSseUrl = mkOption {
      type = types.str;
      example = "http://127.0.0.1:18101/events/main";
      description = ''
        The casper node URL to the SSE events endpoint.
      '';
    };

    casperSyncInterval = mkOption {
      type = types.int;
      default = 10;
      example = 10;
      description = ''
        The interval in seconds to between calls to the casper node to sync the deploys.
      '';
    };

    prover = mkOption {
      description = "Prover server related options";
      default = { };
      type = types.submodule {
        options = {
          protocol = mkOption {
            type = types.enum [ "http" "https" ];
            default = "http";
            description = "The protocol that should be used to connect to the prover server.";
          };

          bindAddress = mkOption {
            type = types.str;
            default = "0.0.0.0";
            example = "0.0.0.0";
            description = ''
              The address of the prover server.
            '';
          };

          port = mkOption {
            type = types.port;
            default = 60001;
            example = 60001;
            description = ''
              The port of the prover server.
            '';
          };

          maxBatchSize = mkOption {
            type = types.nullOr types.ints.unsigned;
            default = null;
            example = 100;
            description = ''
              The maximal amount of transactions that should be included in a batch.
            '';
          };

          maxBatchDuration = mkOption {
            type = types.nullOr types.ints.unsigned;
            default = null;
            example = 180;
            description = ''
              The maximal duration in seconds until batch is created.
            '';
          };
        };
      };
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
        after = [ "network-online.target" "kairos-prover.service" ];
        requires = [ "network-online.target" "kairos-prover.service" ];
        environment = {
          RUST_LOG = cfg.logLevel;
          KAIROS_SERVER_SOCKET_ADDR = "${cfg.bindAddress}:${builtins.toString cfg.port}";
          KAIROS_SERVER_CASPER_RPC = cfg.casperRpcUrl;
          KAIROS_SERVER_CASPER_SSE = cfg.casperSseUrl;
          KAIROS_SERVER_CASPER_SYNC_INTERVAL = cfg.casperSyncInterval;
          KAIROS_SERVER_DEMO_CONTRACT_HASH = cfg.demoContractHash;
          KAIROS_PROVER_SERVER_URL = "${cfg.prover.protocol}://${cfg.prover.bindAddress}:${builtins.toString cfg.prover.port}";
        } // optionalAttrs (!builtins.isNull cfg.prover.maxBatchSize) {
          KAIROS_SERVER_MAX_BATCH_SIZE = cfg.maxBatchSize;
        } // optionalAttrs (!builtins.isNull cfg.prover.maxBatchDuration) {
          KAIROS_SERVER_MAX_BATCH_SECONDS = cfg.prover.maxBatchDuration;
        };
        serviceConfig = mkMerge [
          {
            ExecStart = ''${lib.getExe cfg.package}'';
            Restart = "always";
            DynamicUser = true;
          }
        ];
      };

    services.kairos-prover = {
      enable = true;
      inherit (cfg.prover) bindAddress port;
    };
  };
}
