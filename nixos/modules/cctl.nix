{ lib, config, pkgs, ... }:
let
  inherit (lib)
    types
    mkOption
    mkIf
    mkMerge
    mkEnableOption
    escapeShellArgs
    optionals
    optional
    concatLines
    ;
  cfg = config.services.cctl;
in
{
  options.services.cctl = {

    enable = mkEnableOption "cctl";

    package = mkOption {
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
      type = types.str;
      default = "/var/lib/cctl";
      description = ''
        The working directory path where cctl will put its assets and resources.
      '';
    };

    chainspec = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = ''
        The path to a chainspec.toml.
      '';
    };

    config = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = ''
        The path to a casper node config.toml.
      '';
    };

    logLevel = mkOption {
      type = types.str;
      default = "info";
      description = ''
        The log-level that should be used.
      '';
    };
  };

  config = mkIf cfg.enable {

    systemd.services.cctl =
      let
        writeableChainspec = "${cfg.workingDirectory}/chainspec.toml";
        writeableConfig = "${cfg.workingDirectory}/config.toml";
        args = escapeShellArgs ([
          "--working-dir"
          cfg.workingDirectory
        ]
        ++ optionals (!builtins.isNull cfg.chainspec) [
          "--chainspec-path"
          cfg.chainspec
        ]
        ++ optionals (!builtins.isNull cfg.config) [
          "--config-path"
          cfg.config
        ]);
      in
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
              ExecStartPre =
                concatLines
                  ((optional (!builtins.isNull cfg.chainspec) "${pkgs.coreutils}/bin/cp --no-preserve=mode ${cfg.chainspec} ${writeableChainspec}") ++
                    (optional (!builtins.isNull cfg.config) "${pkgs.coreutils}/bin/cp --no-preserve=mode ${cfg.config} ${writeableConfig}"));
              ExecStart = "${lib.getExe cfg.package} ${args}";
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
      virtualHosts."${config.networking.hostName}".locations."/cctl/users/" = {
        alias = "${cfg.workingDirectory}/assets/users/";
        extraConfig = ''
          autoindex on;
          add_header Content-Type 'text/plain charset=UTF-8';
        '';
      };
    };
  };
}
