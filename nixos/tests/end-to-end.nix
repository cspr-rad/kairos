{ nixosTest
, mkKairosHostConfig
, kairos
, testResources ? ../../kairos-cli/tests/fixtures
, kairos-contracts
, cctlModule
, fetchurl
, casper-client-rs
}:
nixosTest {
  name = "kairos e2e test";

  nodes = {
    server = { config, lib, ... }: {
      imports = [
        (mkKairosHostConfig "kairos")
        cctlModule
      ];

      # modify acme for nixos-test environment
      security.acme = {
        preliminarySelfsigned = true;
        defaults.server = "https://example.com"; # don't spam the acme production server
      };
      environment.systemPackages = [ casper-client-rs ];
      # allow HTTP for nixos-test environment
      services.nginx.virtualHosts.${config.networking.hostName}.forceSSL = lib.mkForce false;

      services.cctl = {
        enable = true;
        chainspec = fetchurl {
          url = "https://raw.githubusercontent.com/cspr-rad/casper-node/kairos-testing-chainspec/resources/local/chainspec.toml.in";
          hash = "1yxrq8zim5gmvq07l0ihakhrprwzhd9fd81h7idd5zkgj390wnif";
        };
      };
      services.kairos.casperRpcUrl = "http://localhost:${builtins.toString config.services.cctl.port}/rpc";
    };

    client = { pkgs, ... }: {
      environment.systemPackages = [ pkgs.curl pkgs.wget kairos ];
    };
  };

  testScript = { nodes, ... }:
    let
      casperNodeAddress = "http://localhost:${builtins.toString nodes.server.services.cctl.port}";
      serverUsersDirectory = nodes.server.services.cctl.workingDirectory + "/assets/users";
      # This is where wget will place the files from http://kairos/cctl/users
      clientUsersDirectory = "kairos/cctl/users";
    in
    ''
      # import json

      start_all()

      kairos.wait_for_unit("cctl.service")

      kairos.wait_for_unit("kairos.service")
      kairos.wait_for_unit("nginx.service")
      kairos.wait_for_open_port(80)

      client.wait_for_unit ("multi-user.target")

      # We need to copy the generated assets from the server to our client
      # For more details, see cctl module implementation
      client.succeed("wget --no-parent -r http://kairos/cctl/users/")

      kairos.succeed("casper-client get-node-status --node-address ${casperNodeAddress}")

      # Deploy the demo contract
      # chain-name see: https://github.com/casper-network/cctl/blob/745155d080934c409d98266f912b8fd2b7e28a00/utils/constants.sh#L66
      kairos.succeed("casper-client put-deploy --node-address ${casperNodeAddress} --chain-name cspr-dev-cctl --secret-key ${serverUsersDirectory}/user-1/secret_key.pem --payment-amount 5000000000000  --session-path ${kairos-contracts}/bin/demo-contract-optimized.wasm")

      # CLI with ed25519
      cli_output = client.succeed("kairos-cli --kairos-server-address http://kairos deposit --amount 1000 --private-key ${clientUsersDirectory}/user-1/secret_key.pem")
      assert int(cli_output, 16), "The deposit command did not output a hex encoded deploy hash. The output was {}".format(cli_output)

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

