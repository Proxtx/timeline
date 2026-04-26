{
  description = "Timeline — life-logging server, frontend bundle, and per-plugin packages.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Stable rust + wasm32 target. The pre-rework Nix build pinned
        # nightly-via-overlay; we no longer need that.
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonNativeBuildInputs = [
          pkgs.pkg-config
          pkgs.openssl
        ];

        # The timeline crates form an implicit workspace via path
        # dependencies (server -> timeline_plugin_sdk -> types, etc.).
        # Crane needs every crate root + Cargo.toml + Cargo.lock to be
        # in the source view, so we hand it the whole timeline directory
        # but filter out target/, dist/, and other build output.
        timelineSrc = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            let
              base = baseNameOf path;
            in
              base != "target"
              && base != "dist"
              && base != "node_modules"
              && (craneLib.filterCargoSources path type
                  || pkgs.lib.hasSuffix ".html" path
                  || pkgs.lib.hasSuffix ".css" path
                  || pkgs.lib.hasSuffix "Trunk.toml" path
                  || pkgs.lib.hasSuffix "rust-toolchain.toml" path);
        };

        # ----- main timeline server -----

        timelineServer = craneLib.buildPackage {
          pname = "timeline-server";
          version = "0.2.0";
          src = timelineSrc;
          cargoExtraArgs = "--locked --bin server -p server";
          cargoLock = ./server/Cargo.lock;
          nativeBuildInputs = commonNativeBuildInputs;
          doCheck = false;
        };

        # ----- frontend trunk bundle -----
        # Trunk runs from frontend/, builds the wasm binary, and emits
        # dist/. The build is two phases: (1) compile the wasm via cargo
        # for the wasm32 target, (2) trunk packages it. We let trunk
        # drive both.
        frontendBundle = pkgs.stdenv.mkDerivation {
          pname = "timeline-frontend";
          version = "0.2.0";
          src = timelineSrc;
          nativeBuildInputs = [
            rustToolchain
            pkgs.trunk
            pkgs.wasm-bindgen-cli
            pkgs.binaryen
          ];
          buildPhase = ''
            cd frontend
            export CARGO_HOME=$(mktemp -d)
            trunk build --release --offline --no-autoreload || trunk build --release
          '';
          installPhase = ''
            mkdir -p $out
            cp -r dist/* $out/
          '';
          dontStrip = true;
        };

      in
      {
        packages = {
          server = timelineServer;
          frontend = frontendBundle;
          default = timelineServer;
        };

        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.trunk
            pkgs.wasm-bindgen-cli
            pkgs.binaryen
            pkgs.pkg-config
            pkgs.openssl
            pkgs.sqlite
          ];
        };
      })
    // {
      # ----------------- NixOS module -----------------
      # Per the Stage 6 design: one systemd unit for the main server, plus
      # one per registered plugin. The user supplies the per-plugin recipe
      # (path to the plugin binary, port, token, plugin-specific config) via
      # `services.timeline.plugins`. The module wires everything together.

      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.services.timeline;

          # Render the main server's config.toml from the NixOS options.
          serverConfigToml = pkgs.writeText "config.toml" ''
            port = ${toString cfg.port}
            password = "${cfg.password}"
            data_dir = "${cfg.dataDir}"
            ${lib.optionalString (cfg.errorReportUrl != null) ''
              error_report_url = "${cfg.errorReportUrl}"
            ''}
            ${lib.concatMapStringsSep "\n" (p: ''
              [[plugin]]
              name = "${p.name}"
              url = "${p.url}"
              token = "${p.token}"
            '') cfg.plugins}
          '';
        in
        {
          options.services.timeline = with lib; {
            enable = mkEnableOption "Timeline";
            package = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.server;
              description = "Timeline server package.";
            };
            frontendDist = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.frontend;
              description = "Static frontend bundle that the server serves.";
            };
            user = mkOption {
              type = types.str;
              default = "timeline";
            };
            group = mkOption {
              type = types.str;
              default = "timeline";
            };
            port = mkOption {
              type = types.port;
              default = 8002;
            };
            password = mkOption {
              type = types.str;
              description = ''
                Cookie password. Plain text in /nix/store — for production
                consider rendering config.toml via systemd's LoadCredential
                instead.
              '';
            };
            dataDir = mkOption {
              type = types.path;
              default = "/var/lib/timeline";
              description = ''
                Holds plugin web bundles under <dataDir>/plugin_web/<name>/
                and main-server runtime files. Each plugin manages its own
                <plugin>-side state under that plugin's data_dir, which can
                be the same path or a different one.
              '';
            };
            errorReportUrl = mkOption {
              type = types.nullOr types.str;
              default = null;
            };
            plugins = mkOption {
              default = [ ];
              description = "List of plugins to register and run.";
              type = types.listOf (types.submodule ({ ... }: {
                options = {
                  name = mkOption { type = types.str; };
                  url = mkOption {
                    type = types.str;
                    description = "Where the plugin listens (e.g. http://127.0.0.1:9001).";
                  };
                  token = mkOption {
                    type = types.str;
                    description = "Shared bearer token. Same value goes into the plugin's own config.toml.";
                  };
                  package = mkOption {
                    type = types.package;
                    description = "Built plugin server binary.";
                  };
                  webBundle = mkOption {
                    type = types.nullOr types.package;
                    default = null;
                    description = "Optional trunk-built client bundle (linked into <dataDir>/plugin_web/<name>/).";
                  };
                  configToml = mkOption {
                    type = types.path;
                    description = "Path to this plugin's own config.toml.";
                  };
                };
              }));
            };
          };

          config = lib.mkIf cfg.enable {
            users.groups.${cfg.group} = { };
            users.users.${cfg.user} = {
              group = cfg.group;
              isSystemUser = true;
            };

            # Lay down the data directories.
            systemd.tmpfiles.rules = [
              "d ${cfg.dataDir} 0770 ${cfg.user} ${cfg.group} -"
              "d ${cfg.dataDir}/plugin_web 0770 ${cfg.user} ${cfg.group} -"
            ] ++ map (p:
              "L+ ${cfg.dataDir}/plugin_web/${p.name} - - - - ${p.webBundle}"
            ) (lib.filter (p: p.webBundle != null) cfg.plugins);

            # Main timeline server.
            systemd.services.timeline = {
              description = "Timeline main server";
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" ];
              serviceConfig = {
                Type = "simple";
                User = cfg.user;
                Group = cfg.group;
                WorkingDirectory = cfg.dataDir;
                ExecStartPre = "${pkgs.coreutils}/bin/cp -f ${serverConfigToml} ${cfg.dataDir}/config.toml";
                ExecStart = "${cfg.package}/bin/server";
                Restart = "always";
                # Make the bundled frontend available to the server's
                # FileServer via a stable relative path.
                BindPaths = [ "${cfg.frontendDist}:${cfg.dataDir}/../frontend/dist" ];
              };
            };

            # One systemd unit per plugin.
            systemd.services = lib.listToAttrs (map (p: {
              name = "timeline-plugin-${p.name}";
              value = {
                description = "Timeline plugin: ${p.name}";
                wantedBy = [ "multi-user.target" ];
                after = [ "network.target" ];
                serviceConfig = {
                  Type = "simple";
                  User = cfg.user;
                  Group = cfg.group;
                  WorkingDirectory = "${cfg.dataDir}/plugins/${p.name}";
                  ExecStartPre = [
                    "${pkgs.coreutils}/bin/mkdir -p ${cfg.dataDir}/plugins/${p.name}"
                    "${pkgs.coreutils}/bin/cp -f ${p.configToml} ${cfg.dataDir}/plugins/${p.name}/config.toml"
                  ];
                  ExecStart = "${p.package}/bin/${p.name}_server";
                  Restart = "always";
                };
              };
            }) cfg.plugins);
          };
        };
    };
}
