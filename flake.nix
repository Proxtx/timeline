{
  inputs = {
    nixpkgs.url    = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url      = "github:ipetkov/crane";
    linker.url     = "path:/home/nixos/timeline/linker/";
  };

  outputs = { self, nixpkgs, flake-utils, crane, linker }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs     = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
      in {
        packages.default = config: let
          plugins = config.plugins;
          experiences = builtins.fetchGit "https://github.com/Proxtx/experiences";
          plugin_repos = builtins.concatStringsSep "\n" (builtins.map (plugin: "cp -r ${(builtins.fetchGit plugin.url).outPath}/ $out/timeline/plugins/${plugin.name}") plugins);
          modifiedSource = pkgs.runCommand "modified-source" {
            buildInputs = [
              pkgs.git
              pkgs.cargo
              linker.packages.${system}.default
            ];
          } ''
            # Create the base directory for the modified source.
            mkdir -p $out

            # Copy the entire repository into $out.
            cp -r ${./.} $out/timeline
            rm -rf $out/git
            chmod -R u+w $out

            # Clone the experiences repository into $out/experiences and record its location.
            cp -r ${experiences} $out/experiences 
            echo -n "../experiences/" > $out/timeline/experiences_location.txt

            # Create the plugins directory and clone each plugin.
            mkdir -p $out/timeline/plugins
            ${plugin_repos}

            # Run the linker command from within the linker folder.
            cd $out/timeline/linker && linker
          '';
        in craneLib.buildPackage {
          # Use the preprocessed source as the input.
          src = "${modifiedSource}/timeline/server";
          # Provide the absolute path to the Cargo.toml file.
          # cargoToml = "${modifiedSource}/timeline/server/Cargo.toml";
          nativeBuildInputs = [
            pkgs.git
            pkgs.cargo
            linker.packages.${system}.default
          ];

          cargoVendorDir = null;
        };
      });
}