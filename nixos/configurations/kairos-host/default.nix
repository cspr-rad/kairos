{ pkgs, config, lib, ... }:
{
  imports = [
    ../common
  ];

  networking.nameservers = [ "8.8.8.8" "8.8.4.4" ];
  networking.firewall.allowedTCPPorts = [ 80 443 ];

  services.nginx = {
    enable = true;
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    virtualHosts.${config.networking.hostName} = {
      forceSSL = true;
      enableACME = true;
      locations = {
        "/api" = {
          proxyPass = "http://${config.services.kairos.bindAddress}:${toString config.services.kairos.port}";
        };
      };
    };
  };

  services.kairos = {
    enable = true;
    demoContractHash = "";
  };
}
