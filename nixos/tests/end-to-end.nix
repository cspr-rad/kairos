{ nixosTest
, mkKairosHostConfig
}:
nixosTest {
  name = "kairos e2e test";

  nodes = {
    server = { config, pkgs, lib, ... }: {
      imports = [
        (mkKairosHostConfig "kairos")
      ];

      # modify acme for nixos-test environment
      security.acme = {
        preliminarySelfsigned = true;
        defaults.server = "https://example.com"; # don't spam the acme production server
      };
    };

    client = { config, pkgs, nodes, ... }: {
      environment.systemPackages = [ pkgs.curl ];
    };
  };

  testScript = ''
    import json

    start_all()

    kairos.wait_for_unit("kairos.service")
    kairos.wait_for_unit("nginx.service")
    kairos.wait_for_open_port(80)

    client.wait_for_unit ("multi-user.target")

    deposit_request = { "public_key": "publickey", "amount": 10 }
    client.succeed("curl -X POST http://kairos/api/v1/deposit -H 'Content-Type: application/json' -d '{}'".format(json.dumps(deposit_request)))

    transfer_request = { "from": "publickey", "signature": "signature", "to": "publickey", "amount": 10 }
    client.succeed("curl -X POST http://kairos/api/v1/transfer -H 'Content-Type: application/json' -d '{}'".format(json.dumps(transfer_request)))

    withdraw_request = { "public_key": "publickey", "signature": "signature", "amount": 10 }
    client.succeed("curl -X POST http://kairos/api/v1/withdraw -H 'Content-Type: application/json' -d '{}'".format(json.dumps(withdraw_request)))
  '';
}
