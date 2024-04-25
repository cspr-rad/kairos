{ nixosTest
, mkKairosHostConfig
, kairos
, testResources ? ../../kairos-cli/tests/fixtures
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

  testScript = ''
    import json

    start_all()

    kairos.wait_for_unit("cctl.service")

    kairos.wait_for_unit("kairos.service")
    kairos.wait_for_unit("nginx.service")
    kairos.wait_for_open_port(80)

    client.wait_for_unit ("multi-user.target")

    kairos.succeed("casper-client get-node-status --node-address http://localhost:11101")

    # TODO: use real public keys and signatures
    alice_public_key = "alice_public_key"
    bob_public_key = "bob_public_key"

    # Tx Payload
    #   nonce = 0
    #   deposit:
    #     amount = 1000
    #
    deposit_payload = "3009020100a004020203e8"
    deposit_request = { "public_key": alice_public_key, "payload": deposit_payload, "signature": "alice_sig_1_todo" }
    # REST API
    client.succeed("curl --fail-with-body -X POST http://kairos/api/v1/deposit -H 'Content-Type: application/json' -d '{}'".format(json.dumps(deposit_request)))

    # Tx Payload
    #   nonce = 0
    #   transfer:
    #     recipient = bob_public_key
    #     amount = 1000
    #
    transfer_payload = "3019020100a114040e626f625f7075626c69635f6b6579020203e8"
    transfer_request = { "public_key": alice_public_key, "payload": transfer_payload, "signature": "alice_sig_2_todo" }
    client.succeed("curl --fail-with-body -X POST http://kairos/api/v1/transfer -H 'Content-Type: application/json' -d '{}'".format(json.dumps(transfer_request)))

    # Tx Payload
    #   nonce = 1
    #   withdrawal:
    #     amount = 1000
    #
    withdraw_payload = "3009020101a204020203e8"
    withdraw_request = { "public_key": bob_public_key, "payload": withdraw_payload, "signature": "bob_sig_1_todo" }
    client.succeed("curl --fail-with-body -X POST http://kairos/api/v1/withdraw -H 'Content-Type: application/json' -d '{}'".format(json.dumps(withdraw_request)))

    # CLI with ed25519
    cli_output = client.succeed("kairos-cli deposit --amount 1000 --private-key ${testResources}/ed25519/secret_key.pem")
    assert "ok\n" in cli_output

    cli_output = client.succeed("kairos-cli transfer --recipient '01a26419a7d82b2263deaedea32d35eee8ae1c850bd477f62a82939f06e80df356' --amount 1000 --private-key ${testResources}/ed25519/secret_key.pem")
    assert "ok\n" in cli_output

    cli_output = client.succeed("kairos-cli withdraw --amount 1000 --private-key ${testResources}/ed25519/secret_key.pem")
    assert "ok\n" in cli_output

    # CLI with secp256k1
    cli_output = client.succeed("kairos-cli deposit --amount 1000 --private-key ${testResources}/secp256k1/secret_key.pem")
    assert "ok\n" in cli_output

    cli_output = client.succeed("kairos-cli transfer --recipient '01a26419a7d82b2263deaedea32d35eee8ae1c850bd477f62a82939f06e80df356' --amount 1000 --private-key ${testResources}/secp256k1/secret_key.pem")
    assert "ok\n" in cli_output

    cli_output = client.succeed("kairos-cli withdraw --amount 1000 --private-key ${testResources}/secp256k1/secret_key.pem")
    assert "ok\n" in cli_output
  '';
}

