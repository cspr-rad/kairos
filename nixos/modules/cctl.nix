{ pkgs, lib, config, ... }:
let
  inherit (lib)
    types
    mkOption
    mkIf
    mkMerge
    mkEnableOption
    ;
  cfg = config.services.cctl;

  casperNodeAddress = "http://127.0.0.1:${builtins.toString cfg.port}";
  deployedContractsDirectory = cfg.workingDirectory + "/contracts";
  deployedContractDestination = deployedContractsDirectory + "/${builtins.baseNameOf cfg.contract}";
  # TODO implement this in kairos-test-utils
  deployContractScript = pkgs.writeShellApplication {
    name = "deploy-contract";
    runtimeInputs = [ cfg.casper-client-package pkgs.jq pkgs.gawk ];
    text = ''
      echo "Deploying contract ${builtins.baseNameOf cfg.contract}"
      DEPLOY_HASH=$(casper-client put-deploy \
        --node-address ${casperNodeAddress} \
        --chain-name cspr-dev-cctl \
        --secret-key ${cfg.workingDirectory}/assets/users/user-1/secret_key.pem \
        --payment-amount 5000000000000  \
        --session-path ${cfg.contract} | jq -r ".result.deploy_hash")
      echo "Waiting for successful execution of deploy"
      max_retries=10
      retry_count=0
      while [[ $retry_count -lt $max_retries ]]; do
        EXECUTION_RESULTS=$(casper-client get-deploy \
          --node-address ${casperNodeAddress} \
          "$DEPLOY_HASH" | jq -r ".result.execution_results")
        if ! echo "$EXECUTION_RESULTS" | jq -e . >/dev/null 2>&1; then
          echo "Invalid JSON received:"
          echo "$EXECUTION_RESULTS"
          exit 1
        fi
        # TODO this should check for a success
        # An empty list indicates that the deploy was not processed yet, so we retry
        if [[ "$EXECUTION_RESULTS" != "[]" ]]; then
          break
        fi
        ((retry_count++)) || true
        echo "retrying: $retry_count/$max_retries"
        sleep 3
      done
      echo "Fetching the state root hash"
      STATE_ROOT_HASH=$(casper-client get-state-root-hash \
        --node-address ${casperNodeAddress} | jq -r ".result.state_root_hash")
      ACCOUNT_HASH=$(casper-client account-address \
        --public-key ${cfg.workingDirectory}/assets/users/user-1/public_key.pem)
      echo "Fetching the contract hash"
      CONTRACT_HASH=$(casper-client query-global-state \
          --node-address ${casperNodeAddress} \
          --state-root-hash "$STATE_ROOT_HASH" \
          --key "$ACCOUNT_HASH" \
          --query-path "kairos_contract_package_hash" | jq -r ".result.stored_value.ContractPackage.versions[0].contract_hash" | awk -F"-" '{print $2}')
          # --query_path "$KAIROS_CONTRACT_PACKAGE_HASH"
      mkdir -p ${deployedContractsDirectory}
      touch ${deployedContractDestination}
      echo "$CONTRACT_HASH" > ${deployedContractDestination}
    '';
  };
in
{
  options.services.cctl = {

    enable = mkEnableOption "cctl";

    package = mkOption {
      type = types.package;
    };

    casper-client-package = mkOption {
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
      type = types.path;
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

    contract = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = ''
        The contract that should be deployed once the network is up and ready.
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
              Restart = "no";
              User = "cctl";
              Group = "cctl";
              StateDirectory = builtins.baseNameOf cfg.workingDirectory;
              WorkingDirectory = cfg.workingDirectory;
              ReadWritePaths = [
                cfg.workingDirectory
              ];
            }
            (lib.optionalAttrs (!builtins.isNull cfg.contract) { ExecStartPost = lib.getExe deployContractScript; })
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
      virtualHosts."${config.networking.hostName}".locations = {
        "/cctl/users/" = {
          alias = "${cfg.workingDirectory}/assets/users/";
          extraConfig = ''
            autoindex on;
            add_header Content-Type 'text/plain charset=UTF-8';
          '';
        };
      } // (lib.optionalAttrs (!builtins.isNull cfg.contract) {
        "/cctl/contracts/" = {
          alias = "${deployedContractsDirectory}/";
          extraConfig = ''
            autoindex on;
            add_header Content-Type 'text/plain charset=UTF-8';
          '';
        };
      });
    };
  };
}
