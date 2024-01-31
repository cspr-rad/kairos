{ nixosTest
, lib
, hostConfiguration
, verifyServices ? [ ]
}:
nixosTest {
  name = "verify host configuration test";

  nodes = {
    server = { config, pkgs, lib, ... }: {
      imports = [
        hostConfiguration
      ];
      networking.hostName = lib.mkForce "server";
    };
  };

  testScript = ''
    start_all()
    ${lib.concatMapStrings (service: ''server.wait_for_unit("'' + service + ''")'') verifyServices}
  '';
}
