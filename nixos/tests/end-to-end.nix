{ nixosTest
, mkKairosHostConfig
, kairos
, kairos-contracts
, cctlModule
, casper-client-rs
, writeShellScript
, casper-chainspec
, casper-node-config
}:
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

      # Required to await successful contract deployment
      environment.systemPackages = [ casper-client-rs ];

      services.cctl = {
        enable = true;
        contract = { "kairos_contract_package_hash" = kairos-contracts + "/bin/demo-contract-optimized.wasm"; };
        chainspec = casper-chainspec;
        config = casper-node-config;
      };

      services.kairos = {
        casperRpcUrl = "http://localhost:${builtins.toString config.services.cctl.port}/rpc";
        casperSseUrl = "http://localhost:18101/events/main"; # has to be hardcoded since cctl is not configurable
        # This is a mandatory module option, we set it to 0. This will be overwritten, see systemd.services.kairos.serviceConfig.ExecStart.
        demoContractHash = "0000000000000000000000000000000000000000000000000000000000000000";
        casperSyncInterval = 2;
      };

      systemd.services.kairos = {
        # We have to wait for cctl to deploy the contract to be able to obtain and export the contract hash
        after = [ "network-online.target" "cctl.service" ];
        requires = [ "network-online.target" "cctl.service" ];
        serviceConfig.ExecStart = lib.mkForce (writeShellScript "start-kairos" ''
          # cctl will write the deployed contract hash to a file whose name is configured by services.cctl.contracts
          export KAIROS_SERVER_DEMO_CONTRACT_HASH=$(cat "${config.services.cctl.workingDirectory}/contracts/kairos_contract_package_hash")
          ${lib.getExe kairos}
        '');
      };
    };

    client = { pkgs, ... }: {
      environment.systemPackages = [ pkgs.wget kairos ];
    };
  };

  extraPythonPackages = p: [ p.backoff ];
  testScript = { nodes }:
    let
      casperNodeAddress = "http://localhost:${builtins.toString nodes.server.config.services.cctl.port}";
      # This is the directory wget will copy to, see script below
      clientUsersDirectory = "kairos/cctl/users";
    in
    ''
      import json
      import backoff

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

      @backoff.on_exception(backoff.expo, Exception, max_tries=5, jitter=backoff.full_jitter)
      def wait_for_transaction(sender, transaction_type, amount):
        transactions_result = client.succeed("kairos-cli --kairos-server-address http://kairos fetch --sender {} --transaction-type {}".format(sender, transaction_type))
        transactions = json.loads(transactions_result)
        if not any(transaction.get("public_key") == sender and transaction.get("amount") == str(amount) for transaction in transactions):
          raise Exception("Couldn't find {} for sender {} with amount {} in transactions\n:{}".format(transaction_type, sender, amount, transactions))

      # Test
      start_all()

      kairos.wait_for_unit("cctl.service")
      kairos.wait_for_unit("kairos.service")
      kairos.wait_for_unit("nginx.service")
      kairos.wait_for_open_port(80)

      client.wait_for_unit ("multi-user.target")

      # We need to copy the cctl generated assets from the server to the client machine,
      # because the invocations of the kairos-cli on the client make use of them.
      # The cctl module enables serving of those generated assets on the server,
      client.succeed("wget --no-parent -r http://kairos/cctl/users/")

      # CLI with ed25519
      # deposit
      deposit_amount = 3000000000 
      depositor = client.succeed("cat ${clientUsersDirectory}/user-2/public_key_hex")
      depositor_private_key = "${clientUsersDirectory}/user-2/secret_key.pem"
      deposit_deploy_hash = client.succeed("kairos-cli --kairos-server-address http://kairos deposit --amount {} --recipient {} --private-key {}".format(deposit_amount, depositor, depositor_private_key))
      assert int(deposit_deploy_hash, 16), "The deposit command did not output a hex encoded deploy hash. The output was {}".format(deposit_deploy_hash)

      wait_for_successful_deploy(deposit_deploy_hash)

      wait_for_transaction(depositor, "deposit", deposit_amount)

      # transfer
      transfer_amount = 1000
      beneficiary = client.succeed("cat ${clientUsersDirectory}/user-3/public_key_hex")
      transfer_output = client.succeed("kairos-cli --kairos-server-address http://kairos transfer --amount {} --recipient {} --private-key {}".format(transfer_amount, beneficiary, depositor_private_key))
      assert "Transfer successfully sent to L2\n" in transfer_output, "The transfer command was not successful: {}".format(transfer_output)

      # data availability
      transactions_result = client.succeed("kairos-cli --kairos-server-address http://kairos fetch --recipient {}".format(beneficiary))
      transactions = json.loads(transactions_result)
      assert any(transaction.get("recipient") == beneficiary and transaction.get("amount") == str(transfer_amount) for transaction in transactions), "Couldn't find the transfer in the L2's DA: {}".format(transactions)

      # withdraw
      withdrawal_amount = 800
      withdrawer = client.succeed("cat ${clientUsersDirectory}/user-3/public_key_hex")
      withdrawer_private_key = "${clientUsersDirectory}/user-3/secret_key.pem"
      withdraw_output = client.succeed("kairos-cli --kairos-server-address http://kairos withdraw --amount {} --private-key {}".format(withdrawal_amount, withdrawer_private_key))
      assert "Withdrawal successfully sent to L2\n" in withdraw_output, "The withdraw command was not successful: {}".format(withdraw_output)

      wait_for_transaction(withdrawer, "withdrawal", withdrawal_amount)

      # TODO cctl does not provide any secp256k1 keys, once support is added it should be tested here
    '';
}

