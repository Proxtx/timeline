{
  inputs = {
    nixpkgs.url    = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url      = "github:ipetkov/crane";
    linker.url     = "github:proxtx/timeline?dir=linker";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, linker, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.nightly.latest.default);
      in {
        packages.default = config: let
          plugins = config.plugins;
          experiencesEnabled = config.experiencesEnabled;
          experiences = builtins.fetchGit config.experiences;
          workspace = ''workspace = {resolver = \"2\", members = [\"timeline/server\",\"timeline/server_api\",\"timeline/link\",\"timeline/types\",\"timeline/link_proc_macro\",${builtins.concatStringsSep "," (builtins.map (plugin: "\\\"timeline/plugins/${plugin.name}/server\\\"") plugins)}]}'';
          plugin_repos = builtins.concatStringsSep "\n" (builtins.map (plugin: "cp -r ${(builtins.fetchGit plugin.git).outPath}/ $out/timeline/plugins/${plugin.name}") plugins);
          plugins_list = builtins.concatStringsSep "," (builtins.map (plugin: plugin.name) plugins);
          modifiedSource = pkgs.runCommand "modified-source" {
            buildInputs = [
              pkgs.git
              pkgs.cargo
              pkgs.cacert
              linker.packages.${system}.default
            ];
          } ''
            # Create the base directory for the modified source.
            mkdir -p $out

            # Copy the entire repository into $out.
            cp -r ${./.} $out/timeline
            chmod -R u+w $out
            echo ${workspace} > $out/Cargo.toml

            # Clone the experiences repository into $out/experiences and record its location.
            cp -r ${experiences} $out/experiences 
            echo -n "../experiences/" > $out/timeline/experiences_location.txt

            # Create the plugins directory and clone each plugin.
            mkdir -p $out/timeline/plugins
            ${plugin_repos}

            echo ${plugins_list} > $out/plugins.txt

            # Run the linker command from within the linker folder.
            cd $out/timeline/linker && linker disable
            cd ..
            cargo generate-lockfile
          '';
          # cargoArtifacts = craneLib.buildDepsOnly {src = "${modifiedSource}/"; cargoExtraArgs = ""; nativeBuildInputs = [pkgs.pkg-config pkgs.openssl];}; 
        in craneLib.buildPackage {
          # inherit cargoArtifacts;
          # Use the preprocessed source as the input.
          src = builtins.trace "${modifiedSource}" "${modifiedSource}/";
          
          doCheck = false;
          # Provide the absolute path to the Cargo.toml file.
          # cargoToml = "${modifiedSource}/timeline/server/Cargo.toml";
          nativeBuildInputs = [
            pkgs.git
            pkgs.cargo
            pkgs.pkg-config 
            pkgs.openssl
            linker.packages.${system}.default
          ];
          cargoExtraArgs = if experiencesEnabled then  "--features=experiences" else "";
          # cargoLock = "${modifiedSource}/timeline/server/Cargo.lock";
        };
      }) // {
        nixosModules.default = {config, lib, pkgs, timelineConfig, ...}: {
          options.services.timeline = {
            enable = pkgs.lib.mkEnableOption "Timeline";
            data_dir = lib.mkOption {
              type = lib.types.path;
              default = "/var/lib/timeline/server";
              description = "Sets the current working directory";
            };
            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.system}.default timelineConfig;
              description = "Timeline Package Executable";
            };
          };

          config = lib.mkIf config.services.timeline.enable {
            users.groups = {
              timeline = {};
            };

            users.users = {
              timeline = {
                group = "timeline";
                isSystemUser = true;
              };
            };

            systemd.services.timeline = {
              wantedBy = ["multi-user.target"];
              serviceConfig = {
                ExecStart = "${config.services.timeline.package}/bin/server";
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