{ nixosTest
, mkKairosHostConfig
, kairos
, testResources ? ../../kairos-cli/tests/fixtures
, kairos-contracts
, cctlModule
, casper-client-rs
, writeShellScript
, jq
}:
let
  # This is where wget will place the files advertised at http://kairos/cctl/users if the cctl module is enabled
  clientUsersDirectory = "kairos/cctl/users";
  clientContractsDirectory = "kairos/cctl/contracts";
  cctlPort = 11101;
  casperNodeAddress = "http://localhost:${builtins.toString cctlPort}";
  cctlWorkingDirectory = "/var/lib/cctl";
  serverUsersDirectory = cctlWorkingDirectory + "/assets/users";
  serverContractsDirectory = cctlWorkingDirectory + "/contracts";
  # chain-name see: https://github.com/casper-network/cctl/blob/745155d080934c409d98266f912b8fd2b7e28a00/utils/constants.sh#L66
  casperChainName = "cspr-dev-cctl";
in
nixosTest {
  name = "kairos e2e test";
  nodes = {
    server = { config, lib, ... }: {
      imports = [
        (mkKairosHostConfig "kairos")
        cctlModule
      ];

      # allow HTTP for nixos-test environment
      services.nginx.virtualHosts.${config.networking.hostName} = {
        forceSSL = lib.mkForce false;
        enableACME = lib.mkForce false;
      };

      environment.systemPackages = [ casper-client-rs jq kairos-contracts ];

      services.cctl = {
        enable = true;
        port = cctlPort;
        workingDirectory = cctlWorkingDirectory;
        contract = kairos-contracts + "/bin/demo-contract-optimized.wasm";
      };

      services.kairos = {
        casperRpcUrl = "http://localhost:${builtins.toString config.services.cctl.port}/rpc";
        casperSseUrl = "http://127.0.0.1:18101/events/main";
        demoContractHash = "";
      };

      systemd.services.kairos = {
        path = [ casper-client-rs jq ];
        after = [ "network-online.target" "cctl.service" ];
        requires = [ "network-online.target" "cctl.service" ];
        serviceConfig.ExecStart = lib.mkForce (writeShellScript "start-kairos" ''
          # Deploy the demo contract and export the environment variable
          # such that the kairos-server gets configured accordingly
          export KAIROS_SERVER_DEMO_CONTRACT_HASH=${serverContractsDirectory}
          ${lib.getExe kairos}
        '');
      };
    };

    client = { pkgs, ... }: {
      environment.systemPackages = [ pkgs.curl pkgs.wget kairos ];
    };
  };

  testScript = ''
    import json

    start_all()

    kairos.wait_for_unit("cctl.service")

    kairos.wait_for_unit("kairos.service")
    kairos.wait_for_unit("nginx.service")
    kairos.wait_for_open_port(80)

    client.wait_for_unit ("multi-user.target")

    # We need to copy the generated assets from the server to our client
    # For more details, see cctl module implementation
    client.succeed("wget --no-parent -r http://kairos/cctl/users/")
    # Copying the served contracts for debugging purposes
    client.succeed("wget --no-parent -r http://kairos/cctl/contracts/")

    contract_hash = kairos.succeed("cat ${serverContractsDirectory}/demo-contract-optimized.wasm")

    kairos.succeed("casper-client get-node-status --node-address ${casperNodeAddress}")

    # CLI with ed25519
    deploy_hash = client.succeed("kairos-cli --kairos-server-address http://kairos deposit --amount 3000000000 --private-key ${clientUsersDirectory}/user-2/secret_key.pem --contract-hash {}".format(contract_hash))
    assert int(deploy_hash, 16), "The deposit command did not output a hex encoded deploy hash. The output was {}".format(deploy_hash)

    get_deploy_result = { "result": { "execution_results": [ "" ] } }
    while "Success" not in get_deploy_result["result"]["execution_results"]:
      client_output = kairos.succeed("casper-client get-deploy --node-address ${casperNodeAddress} {}".format(deploy_hash))
      get_deploy_result = json.loads(client_output)
      print("GIGGI {}".format(client_output))
    
    assert "hello" in client_output, "Not processed yet {}".format(client_output)

    # TODO Transfer and withdraw can only work once deposit deploys are processed and the users actually have an account
    # REST API
    # Tx Payload
    #   nonce = 0
    #   transfer:
    #     recipient = deadbabe
    #     amount = 1000
    #
    # transfer_payload = "300f020100a10a0404deadbabe020203e8"
    # transfer_request = { "public_key": "cafebabe", "payload": transfer_payload, "signature": "deadbeef" }
    # client.succeed("curl --fail-with-body -X POST http://kairos/api/v1/transfer -H 'Content-Type: application/json' -d '{}'".format(json.dumps(transfer_request)))

    # Tx Payload
    #   nonce = 0
    #   withdrawal:
    #     amount = 1000
    #
    # withdraw_payload = "3009020100a204020203e8"
    # withdraw_request = { "public_key": "deadbabe", "payload": withdraw_payload, "signature": "deadbeef" }
    # client.succeed("curl --fail-with-body -X POST http://kairos/api/v1/withdraw -H 'Content-Type: application/json' -d '{}'".format(json.dumps(withdraw_request)))

    # TODO Transfer and withdraw can only work once deposit deploys are processed and the users actually have an account
    # CLI with ed25519
    # cli_output = client.succeed("kairos-cli transfer --recipient '01a26419a7d82b2263deaedea32d35eee8ae1c850bd477f62a82939f06e80df356' --amount 1000 --private-key ${testResources}/ed25519/secret_key.pem")
    # assert "ok\n" in cli_output

    # cli_output = client.succeed("kairos-cli withdraw --amount 1000 --private-key ${testResources}/ed25519/secret_key.pem")
    # assert "ok\n" in cli_output

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

