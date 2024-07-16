{ nixosTest
, mkKairosHostConfig
, kairos
, testResources ? ../../kairos-cli/tests/fixtures
, kairos-contracts
, cctlModule
, fetchurl
, casper-client-rs
, writeShellScript
, jq
}:
let
  # This is where wget (see test) will place the files advertised at http://kairos/cctl/users if the cctl module is enabled
  clientUsersDirectory = "kairos/cctl/users";
  cctlPort = 11101;
  casperNodeAddress = "http://localhost:${builtins.toString cctlPort}";
  cctlWorkingDirectory = "/var/lib/cctl";
  contractHashName = "kairos_contract_package_hash";
  # The path where cctl will write the deployed contract hash on the servers filesystem
  serverContractHashPath = "${cctlWorkingDirectory}/contracts/${contractHashName}";
in
nixosTest {
  name = "kairos e2e test";
  nodes = {
    server = { config, lib, ... }: {
      imports = [
        (mkKairosHostConfig "kairos")
        cctlModule
      ];

      virtualisation.cores = 4;
      virtualisation.memorySize = 4096;

      # allow HTTP for nixos-test environment
      services.nginx.virtualHosts.${config.networking.hostName} = {
        forceSSL = lib.mkForce false;
        enableACME = lib.mkForce false;
      };

      environment.systemPackages = [ casper-client-rs ];

      services.cctl = {
        enable = true;
        port = cctlPort;
        workingDirectory = cctlWorkingDirectory;
        contract = { "${contractHashName}" = kairos-contracts + "/bin/demo-contract-optimized.wasm"; };
        chainspec = fetchurl {
          url = "https://raw.githubusercontent.com/cspr-rad/casper-node/53136ac5f004f2ae70a75b4eeb2ff7d907aff6aa/resources/local/chainspec.toml.in";
          hash = "sha256-b/6c5o3JXFlaTgTHxs8JepaHzjMG75knzlKKqRd/7pc=";
        };
        config = fetchurl {
          url = "https://raw.githubusercontent.com/cspr-rad/casper-node/53136ac5f004f2ae70a75b4eeb2ff7d907aff6aa/resources/local/config.toml";
          hash = "sha256-ZuNbxw0nBjuONEZRK8Ru96zZQak4MEQ/eM1fA6esyCM=";
        };
      };

      services.kairos = {
        casperRpcUrl = "http://localhost:${builtins.toString config.services.cctl.port}/rpc";
        casperSseUrl = "http://localhost:18101/events/main"; # has to be hardcoded since it's not configurable atm
        casperSyncInterval = 5;
        demoContractHash = "0000000000000000000000000000000000000000000000000000000000000000";
      };

      # We have to wait for cctl to deploy the contract to be able to obtain and export the contract hash
      systemd.services.kairos = {
        path = [ casper-client-rs jq ];
        after = [ "network-online.target" "cctl.service" ];
        requires = [ "network-online.target" "cctl.service" ];
        serviceConfig.ExecStart = lib.mkForce (writeShellScript "start-kairos" ''
          export KAIROS_SERVER_DEMO_CONTRACT_HASH=$(cat ${serverContractHashPath})
          ${lib.getExe kairos}
        '');
      };
    };

    client = { pkgs, ... }: {
      environment.systemPackages = [ pkgs.curl pkgs.wget kairos ];
    };
  };

  extraPythonPackages = p: [ p.backoff ];
  testScript = ''
    import json
    import backoff
    import time

    # Utils
    def verify_deploy_success(json_data):
      # Check if the "Success" key is present
      try:
        if "result" in json_data and "execution_results" in json_data["result"]:
          for execution_result in json_data["result"]["execution_results"]:
            if "result" in execution_result and "Success" in execution_result["result"]:
              return True
      except KeyError:
        pass
      return False

    @backoff.on_exception(backoff.expo, Exception, max_tries=5, jitter=backoff.full_jitter)
    def wait_for_successful_deploy(deploy_hash):
      client_output = kairos.succeed("casper-client get-deploy --node-address ${casperNodeAddress} {}".format(deploy_hash))
      get_deploy_result = json.loads(client_output)
      if not verify_deploy_success(get_deploy_result):
        raise Exception("Success key not found in JSON")

    # Test
    start_all()

    kairos.wait_for_unit("cctl.service")

    kairos.wait_for_unit("kairos.service")
    kairos.wait_for_unit("nginx.service")
    kairos.wait_for_open_port(80)

    client.wait_for_unit ("multi-user.target")

    # We need to copy the generated assets from the server to our client, because we use filepaths
    # in our cli, therefore we need to make sure that the files generated by cctl on the server
    # are also available on the client
    # For more details, see cctl module implementation
    client.succeed("wget --no-parent -r http://kairos/cctl/users/")

    contract_hash = kairos.succeed("cat ${serverContractHashPath}")

    kairos.succeed("casper-client get-node-status --node-address ${casperNodeAddress}")

    # CLI with ed25519
    # deposit
    depositor = client.succeed("cat ${clientUsersDirectory}/user-2/public_key_hex")
    depositor_private_key = "${clientUsersDirectory}/user-2/secret_key.pem"
    deposit_deploy_hash = client.succeed("kairos-cli --kairos-server-address http://kairos deposit --amount 3000000000 --recipient {} --private-key {} --contract-hash {}".format(depositor, depositor_private_key, contract_hash))
    assert int(deposit_deploy_hash, 16), "The deposit command did not output a hex encoded deploy hash. The output was {}".format(deposit_deploy_hash)

    wait_for_successful_deploy(deposit_deploy_hash)

    # wait for l2 to sync with l1 every 10 seconds
    time.sleep(12)


    # transfer
    beneficiary = client.succeed("cat ${clientUsersDirectory}/user-3/public_key_hex")
    # TODO we currently have to pass the nonce explicitly as we are lacking a way to synchronize
    # the successful creation of an account in the account trie after a deposit.
    # After adding the DA and awaiting that the server picked up the new deposit and thus
    # an according account was added to the trie, we can discard the nonce and get it from the server
    transfer_output = client.succeed("kairos-cli --kairos-server-address http://kairos transfer --nonce 0 --amount 1000 --recipient {} --private-key {}".format(beneficiary, depositor_private_key))
    assert "Transfer successfully sent to L2\n" in transfer_output, "The transfer command was not successful: {}".format(transfer_output)

    # TODO test withdraw

    # TODO cctl does not provide any secp256k1 keys
    # CLI with secp256k1
    # cli_output = client.succeed("kairos-cli --kairos-server-address http://kairos deposit --amount 1000 --private-key ${testResources}/secp256k1/secret_key.pem")
    # assert "ok\n" in cli_output

    # cli_output = client.succeed("kairos-cli transfer --recipient '01a26419a7d82b2263deaedea32d35eee8ae1c850bd477f62a82939f06e80df356' --amount 1000 --private-key ${testResources}/secp256k1/secret_key.pem")
    # assert "ok\n" in cli_output

    # cli_output = client.succeed("kairos-cli withdraw --amount 1000 --private-key ${testResources}/secp256k1/secret_key.pem")
    # assert "ok\n" in cli_output
  '';
}

