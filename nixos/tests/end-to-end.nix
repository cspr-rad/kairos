{ nixosTest
, mkKairosHostConfig
, kairos
, testResources ? ../../kairos-cli/tests/fixtures
, kairos-contracts
, cctlModule
, casper-client-rs
}:
nixosTest {
  name = "kairos e2e test";

  nodes = {
    server = { config, pkgs, lib, ... }: {
      imports = [
        (mkKairosHostConfig "kairos")
        cctlModule
      ];

      # modify acme for nixos-test environment
      security.acme = {
        preliminarySelfsigned = true;
        defaults.server = "https://example.com"; # don't spam the acme production server
      };
      services.cctl.enable = true;
      environment.systemPackages = [ casper-client-rs ];
      # allow HTTP for nixos-test environment
      services.nginx.virtualHosts.${config.networking.hostName}.forceSSL = lib.mkForce false;
    };

    client = { config, pkgs, nodes, ... }: {
      environment.systemPackages = [ pkgs.curl kairos ];
    };
  };

  testScript =
    let
      cctlUsersDirectory = "/var/lib/cctl/assets/users";
      # port see: https://github.com/casper-network/cctl/blob/745155d080934c409d98266f912b8fd2b7e28a00/utils/constants.sh#L13
      casperNodeAddress = "http://localhost:11101";
    in
    ''
      import json

      start_all()

      kairos.wait_for_unit("cctl.service")

      kairos.wait_for_unit("kairos.service")
      kairos.wait_for_unit("nginx.service")
      kairos.wait_for_open_port(80)

      client.wait_for_unit ("multi-user.target")

      kairos.succeed("casper-client get-node-status --node-address ${casperNodeAddress}")

      # chain-name see: https://github.com/casper-network/cctl/blob/745155d080934c409d98266f912b8fd2b7e28a00/utils/constants.sh#L66
      kairos.succeed("casper-client put-deploy --node-address ${casperNodeAddress} --chain-name cspr-dev-cctl  --secret-key $(ls ${cctlUsersDirectory}/user-1/secret_key.pem) --payment-amount 5000000000000  --session-path ${kairos-contracts}/bin/demo-contract-optimized.wasm")

      # REST API
      # Tx Payload
      #   nonce = 0
      #   deposit:
      #     amount = 1000
      #
      deposit_payload = "3009020100a004020203e8"
      deposit_request = { "public_key": "cafebabe", "payload": deposit_payload, "signature": "deadbeef" }
      # REST API
      client.succeed("curl --fail-with-body -X POST http://kairos/api/v1/deposit -H 'Content-Type: application/json' -d '{}'".format(json.dumps(deposit_request)))

      # Tx Payload
      #   nonce = 0
      #   transfer:
      #     recipient = deadbabe
      #     amount = 1000
      #
      transfer_payload = "300f020100a10a0404deadbabe020203e8"
      transfer_request = { "public_key": "cafebabe", "payload": transfer_payload, "signature": "deadbeef" }
      client.succeed("curl --fail-with-body -X POST http://kairos/api/v1/transfer -H 'Content-Type: application/json' -d '{}'".format(json.dumps(transfer_request)))

      # Tx Payload
      #   nonce = 0
      #   withdrawal:
      #     amount = 1000
      #
      withdraw_payload = "3009020100a204020203e8"
      withdraw_request = { "public_key": "deadbabe", "payload": withdraw_payload, "signature": "deadbeef" }
      client.succeed("curl --fail-with-body -X POST http://kairos/api/v1/withdraw -H 'Content-Type: application/json' -d '{}'".format(json.dumps(withdraw_request)))

      # CLI with ed25519
      cli_output = client.succeed("kairos-cli --kairos-server-address http://kairos deposit --amount 1000 --private-key ${testResources}/ed25519/secret_key.pem")
      assert "ok\n" in cli_output

      cli_output = client.succeed("kairos-cli transfer --recipient '01a26419a7d82b2263deaedea32d35eee8ae1c850bd477f62a82939f06e80df356' --amount 1000 --private-key ${testResources}/ed25519/secret_key.pem")
      assert "ok\n" in cli_output

      cli_output = client.succeed("kairos-cli withdraw --amount 1000 --private-key ${testResources}/ed25519/secret_key.pem")
      assert "ok\n" in cli_output

      # CLI with secp256k1
      cli_output = client.succeed("kairos-cli --kairos-server-address http://kairos deposit --amount 1000 --private-key ${testResources}/secp256k1/secret_key.pem")
      assert "ok\n" in cli_output

      cli_output = client.succeed("kairos-cli transfer --recipient '01a26419a7d82b2263deaedea32d35eee8ae1c850bd477f62a82939f06e80df356' --amount 1000 --private-key ${testResources}/secp256k1/secret_key.pem")
      assert "ok\n" in cli_output

      cli_output = client.succeed("kairos-cli withdraw --amount 1000 --private-key ${testResources}/secp256k1/secret_key.pem")
      assert "ok\n" in cli_output
    '';
}

