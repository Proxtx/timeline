{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    linker.url = "path:/home/nixos/timeline/linker/";
  };

  outputs = {self, nixpkgs, flake-utils, crane, linker}:
    flake-utils.lib.eachDefaultSystem(system:
      let 
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
      in {
        packages.default = config: let 
          plugins = config.plugins;
          experiences = builtins.fetchGit "https://github.com/Proxtx/experiences";
          in 
          craneLib.buildPackage {
          src = craneLib.cleanCargoSource (let 
          plugin_repos = builtins.concatStringsSep "\n" (builtins.map (plugin: "cp -r ${(builtins.fetchGit plugin.url).outPath}/ $out/timeline/plugins/${plugin.name}") plugins);
          cps = pkgs.runCommand "copy-source" { buildInputs = [ pkgs.git pkgs.cargo linker.packages.${system}.default ]; } ''
                    mkdir -p $out
                    cp -r ${./.} $out/timeline/
                    chmod -R u+w $out
                    mkdir $out/timeline/plugins
                    cp -r ${experiences} $out/experiences
                    echo -n "../experiences/" > $out/timeline/experiences_location.txt
                    ${plugin_repos}

                    cd $out/timeline/linker
                    linker
                  ''; in builtins.trace "${cps.outPath}/server" "${cps.outPath}/timeline/server");

          nativeBuildInputs = [ pkgs.git pkgs.cargo ];

          /*prePatch = ''
            echo "Creating external dependencies folder..."
            mkdir -p plugins

            echo "Cloning repositories..."
            git clone  plugins/

            echo "Running Rust tool to modify files..."
            cd linker
            cargo run .
          '';*/

        };
      }) // {
      /*nixosModules.default = {config, lib, pkgs, ...} : {
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
      };*/
    };
}