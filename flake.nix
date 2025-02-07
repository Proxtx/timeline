{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {self, nixpkgs, flake-utils, crane}:
    flake-utils.lib.eachDefaultSystem(system: 
      let 
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
      in {
        packages.default = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
        };
      }) // {
      nixosModules.default = {config, lib, pkgs, ...} : {
        options.services.timeline = {
          enable = pkgs.lib.mkEnableOption "Timeline";
          config = lib.mkOption {
            type = lib.types.path;
            default = "";
            description = "Configuration for timeline.";
          };
          cwd = lib.mkOption {
            type = lib.types.path;
            default = "/var/lib/timeline/";
            description = "Sets the current working directory";
          };
          package = lib.mkOption {
            type = lib.types.package;
            default = self.packages.${pkgs.system}.default;
            description = "Timeline";
          };
        };

        config = lib.mkIf config.services.tiemline.enable {
          
          users.groups = {
            timeline = {};
          };

          users.users = {
            timeline = {
              group = "timeline";
              isSystemUser = true;
            };
          };
          
          system.activationScripts.copyConfigTimeline = ''
            cp ${config.services.timeline.config} ${config.services.timeline.data_dir}/config.toml
          '';

          systemd.services.timeline = {
            wantedBy = ["multi-user.target"];
            serviceConfig = {
              ExecStart = "${config.services.timeline.package}/bin/timeline";
              Restart = "always";
              User = "timeline";
              WorkingDirectory = "${config.services.timeline.data_dir}";
              Group = "timeline";
            };
          };

          systemd.tmpfiles.settings = {
            "timelineStorage".${config.services.timeline.data_dir}.d = {
              user = "timeline";
              group = "timeline";
              mode = "0770";
            };
          };
        };
      };
    };
}