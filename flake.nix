{
  description = "Timeline — life-logging server, frontend bundle, and plugin build helpers.";

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
    let
      # ---- shared, system-parameterised build toolbox ----
      # Exposed via `self.lib` so each plugin's own flake can build itself
      # against the exact same nixpkgs + rust toolchain (plugins depend on the
      # timeline SDK crates by relative path, so they need the timeline source
      # tree in the build sandbox — see `mergedSrc`).
      mkToolbox = system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          };
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          # Plugins pin wasm-bindgen 0.2.118; trunk needs a byte-matching
          # wasm-bindgen-cli on PATH or it tries to download one (which fails
          # under the build sandbox). nixpkgs ships a newer one, so build 0.2.118.
          wasmBindgenCli = pkgs.buildWasmBindgenCli rec {
            src = pkgs.fetchCrate {
              pname = "wasm-bindgen-cli";
              version = "0.2.118";
              hash = "sha256-ve783oYH0TGv8Z8lIPdGjItzeLDQLOT5uv/jbFOlZpI=";
            };
            cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
              inherit src;
              inherit (src) pname version;
              hash = "sha256-EYDfuBlH3zmTxACBL+sjicRna84CvoesKSQVcYiG9P0=";
            };
          };

          # Timeline source minus build output and the gitignored plugins/.
          # Carries the SDK crates (timeline_plugin_sdk, _client_sdk, types)
          # that every plugin path-depends on.
          timelineSrc = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              let base = baseNameOf path; in
              base != "target"
              && base != "dist"
              && base != "node_modules"
              && base != "plugins"
              && (craneLib.filterCargoSources path type
                  || pkgs.lib.hasSuffix ".html" path
                  || pkgs.lib.hasSuffix ".css" path
                  || pkgs.lib.hasSuffix ".js" path
                  || pkgs.lib.hasSuffix ".svg" path
                  || pkgs.lib.hasSuffix "Trunk.toml" path
                  || pkgs.lib.hasSuffix "rust-toolchain.toml" path);
          };

          cleanPluginSrc = pluginSrc: pkgs.lib.cleanSourceWith {
            src = pluginSrc;
            filter = path: type:
              let base = baseNameOf path; in
              base != "target" && base != "dist" && base != "node_modules";
          };

          # Source of one of the three SDK crates a plugin can path-depend on.
          tlCrate = sub: pkgs.lib.cleanSourceWith {
            src = ./. + "/${sub}";
            filter = path: type:
              let base = baseNameOf path; in base != "target" && base != "dist";
          };

          # Assemble a real Cargo *workspace* with only the crates the plugin's
          # <target> (server|client) needs: `types`, the matching SDK crate, and
          # the plugin's own crate — laid out so the plugins' `../../../` path
          # deps resolve, with the plugin's own Cargo.lock as the workspace lock.
          # A genuine root Cargo.toml + Cargo.lock is what lets crane build the
          # dependency closure once into a cached `cargoArtifacts` derivation, so
          # rebuilds only recompile the plugin's own crates.
          mkWorkspace = { name, pluginSrc, target }:
            let
              sdkMember = if target == "server" then "timeline_plugin_sdk" else "timeline_plugin_client_sdk";
              otherTarget = if target == "server" then "client" else "server";
              members = [ "types" sdkMember "plugins/${name}/${target}" ];
              wsToml = pkgs.writeText "workspace-cargo.toml" ''
                [workspace]
                resolver = "2"
                members = [ ${pkgs.lib.concatMapStringsSep ", " (m: "\"${m}\"") members} ]
                exclude = [ "plugins/${name}/${otherTarget}" ]
              '';
            in
            pkgs.runCommand "timeline-${name}-${target}-src" { } ''
              mkdir -p $out/plugins/${name}
              cp -r ${tlCrate "types"} $out/types
              cp -r ${tlCrate sdkMember} $out/${sdkMember}
              cp -r ${cleanPluginSrc pluginSrc}/. $out/plugins/${name}/
              chmod -R u+w $out
              cp ${pluginSrc + "/${target}/Cargo.lock"} $out/Cargo.lock
              cp ${wsToml} $out/Cargo.toml
            '';

          # The timeline repo has no root workspace; like the plugins, the main
          # server/frontend crates path-depend on `../types`. Build each in a
          # generated workspace containing just the crates it needs, with that
          # crate's own Cargo.lock — so crane can cache the dep closure.
          mkTlWorkspace = { name, members, lock }:
            let
              wsToml = pkgs.writeText "Cargo.toml" ''
                [workspace]
                resolver = "2"
                members = [ ${pkgs.lib.concatMapStringsSep ", " (m: "\"${m}\"") members} ]
              '';
            in
            pkgs.runCommand "timeline-${name}-ws" { } ''
              mkdir -p $out
              ${pkgs.lib.concatMapStringsSep "\n" (m: "cp -r ${tlCrate m} $out/${m}") members}
              chmod -R u+w $out
              cp ${lock} $out/Cargo.lock
              cp ${wsToml} $out/Cargo.toml
            '';

          # ---- main timeline server ----
          timelineServer =
            let
              args = {
                src = mkTlWorkspace { name = "server"; members = [ "types" "server" ]; lock = ./server/Cargo.lock; };
                pname = "timeline-server";
                version = "0.2.0";
                strictDeps = true;
                cargoExtraArgs = "--locked -p server";
                nativeBuildInputs = [ pkgs.pkg-config ];
                buildInputs = [ pkgs.openssl ];
                doCheck = false;
              };
              cargoArtifacts = craneLib.buildDepsOnly args;
            in
            craneLib.buildPackage (args // { inherit cargoArtifacts; });

          # ---- main frontend trunk bundle ----
          frontendBundle =
            let
              args = {
                src = mkTlWorkspace { name = "frontend"; members = [ "types" "frontend" ]; lock = ./frontend/Cargo.lock; };
                pname = "timeline-frontend";
                version = "0.2.0";
                strictDeps = true;
                doCheck = false;
                cargoExtraArgs = "--locked -p timeline_frontend";
                CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
                nativeBuildInputs = [ pkgs.pkg-config ];
              };
              cargoArtifacts = craneLib.buildDepsOnly args;
            in
            craneLib.buildTrunkPackage (args // {
              inherit cargoArtifacts;
              trunkIndexPath = "frontend/index.html";
              wasm-bindgen-cli = wasmBindgenCli;
              nativeBuildInputs = [ pkgs.pkg-config pkgs.binaryen ];
              buildPhaseCargoCommand = ''
                ( cd frontend && trunk build --release=true index.html )
              '';
            });

          # ---- a single plugin: server binary + trunk web bundle (crane) ----
          # Deps compile once into `cargoArtifacts`; the plugin's own crates are
          # the only thing rebuilt when its source changes.
          buildPluginServer = name: pluginSrc:
            let
              args = {
                src = mkWorkspace { inherit name pluginSrc; target = "server"; };
                pname = "${name}_server";
                version = "0.2.0";
                strictDeps = true;
                cargoExtraArgs = "--locked -p ${name}_server";
                nativeBuildInputs = [ pkgs.pkg-config ];
                buildInputs = [ pkgs.openssl ];
                doCheck = false;
              };
              cargoArtifacts = craneLib.buildDepsOnly args;
            in
            craneLib.buildPackage (args // { inherit cargoArtifacts; });

          buildPluginWeb = name: pluginSrc:
            let
              args = {
                src = mkWorkspace { inherit name pluginSrc; target = "client"; };
                pname = "${name}-web";
                version = "0.2.0";
                strictDeps = true;
                doCheck = false;
                cargoExtraArgs = "--locked -p ${name}_client";
                CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
                nativeBuildInputs = [ pkgs.pkg-config ];
              };
              cargoArtifacts = craneLib.buildDepsOnly args;
            in
            craneLib.buildTrunkPackage (args // {
              inherit cargoArtifacts;
              trunkIndexPath = "plugins/${name}/client/index.html";
              wasm-bindgen-cli = wasmBindgenCli;
              nativeBuildInputs = [ pkgs.pkg-config pkgs.binaryen ];
              # Run trunk inside the client member dir — the virtual workspace
              # root has no [package], which trunk rejects. Subshell so the
              # default install still copies plugins/<name>/client/dist.
              buildPhaseCargoCommand = ''
                ( cd plugins/${name}/client && trunk build --release=true index.html )
              '';
            });

          # Combined plugin output: `${plugin}/bin/<name>_server` plus
          # `${plugin}/share/web/` (the trunk dist). The NixOS module reads
          # both from this single derivation, so a plugin flake exposes one
          # package per plugin.
          buildPlugin = { name, src }:
            let
              server = buildPluginServer name src;
              web = buildPluginWeb name src;
            in
            pkgs.symlinkJoin {
              name = "${name}";
              paths = [ server ];
              postBuild = ''
                mkdir -p $out/share
                ln -s ${web} $out/share/web
              '' + pkgs.lib.optionalString (builtins.pathExists (src + "/server/js")) ''
                # documents ships a pdfjs distribution (+ pdfGen.js) the plugin
                # server serves at /js; package it so pdfjs_path can point here.
                cp -r ${src + "/server/js"} $out/share/pdfjs-root
              '';
              passthru = { inherit server web; pluginName = name; };
            };
        in
        {
          inherit pkgs rustToolchain craneLib wasmBindgenCli
            timelineServer frontendBundle buildPlugin buildPluginServer buildPluginWeb;
        };
    in
    flake-utils.lib.eachDefaultSystem (system:
      let tb = mkToolbox system; in
      {
        packages = {
          server = tb.timelineServer;
          frontend = tb.frontendBundle;
          default = tb.timelineServer;
        };

        devShells.default = tb.pkgs.mkShell {
          packages = [
            tb.rustToolchain
            tb.pkgs.trunk
            tb.wasmBindgenCli
            tb.pkgs.binaryen
            tb.pkgs.pkg-config
            tb.pkgs.openssl
            tb.pkgs.sqlite
          ];
        };
      })
    // {
      # A plugin's flake calls this to produce its package:
      #   timeline.lib.buildPlugin { inherit system; name = "..."; src = ./.; }
      lib.buildPlugin = { system, name, src }:
        (mkToolbox system).buildPlugin { inherit name src; };

      # ----------------- NixOS module -----------------
      # One systemd unit for the main server + one per plugin. The whole stack
      # is configured in a single `services.timeline` block: each plugin entry
      # gives its built package, port, token, and an inline `settings` attrset
      # that becomes the plugin's own `[config]` section.
      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.services.timeline;
          tomlFormat = pkgs.formats.toml { };

          # pdfium for the documents plugin: nixpkgs ships a prebuilt
          # libpdfium.so, so this is fully hermetic / sandbox-clean.
          pdfium = pkgs.pdfium-binaries;

          # If an error plugin is configured, point every other plugin's
          # ErrorReporter at its /report endpoint. The error plugin listens on
          # localhost only (plugin ports are never exposed through nginx), so
          # no URL secret is needed.
          errorPlugin = lib.findFirst (p: p.name == "timeline_plugin_error") null cfg.plugins;
          errorUrl =
            if errorPlugin != null
            then "http://127.0.0.1:${toString errorPlugin.port}/report"
            else null;

          # Render one plugin's config.toml from its options.
          pluginConfig = p:
            tomlFormat.generate "${p.name}-config.toml" ({
              plugin = {
                name = p.name;
                port = p.port;
                token = p.token;
                data_dir = cfg.dataDir;
              } // lib.optionalAttrs (p.displayName != null) { display_name = p.displayName; }
                // lib.optionalAttrs (errorUrl != null && p.name != "timeline_plugin_error") {
                  error_report_url = errorUrl;
                };
              config = p.settings
                # documents needs pdfium + pdfjs paths injected automatically.
                // lib.optionalAttrs (p.name == "timeline_plugin_documents") {
                  pdfium_path = "${pdfium}/lib";
                  pdfjs_path = "${p.package}/share/pdfjs-root";
                };
            });

          serverConfigToml = pkgs.writeText "timeline-config.toml" ''
            port = ${toString cfg.port}
            password = "${cfg.password}"
            data_dir = "${cfg.dataDir}"
            ${let u = if cfg.errorReportUrl != null then cfg.errorReportUrl else errorUrl;
              in lib.optionalString (u != null) ''error_report_url = "${u}"''}
            ${lib.concatMapStringsSep "\n" (p: ''
              [[plugin]]
              name = "${p.name}"
              url = "http://127.0.0.1:${toString p.port}"
              token = "${p.token}"
            '') cfg.plugins}
          '';
        in
        {
          options.services.timeline = with lib; {
            enable = mkEnableOption "Timeline life-logging server";
            package = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.server;
              description = "Main timeline server package.";
            };
            frontendDist = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.frontend;
              description = "Static frontend bundle the server serves.";
            };
            user = mkOption { type = types.str; default = "timeline"; };
            group = mkOption { type = types.str; default = "timeline"; };
            port = mkOption { type = types.port; default = 8002; };
            password = mkOption {
              type = types.str;
              description = ''
                Cookie login password. Ends up world-readable in the Nix store;
                for real secrets use sops-nix / agenix / systemd LoadCredential.
              '';
            };
            dataDir = mkOption {
              type = types.str;
              default = "/var/lib/timeline";
              description = "Holds per-plugin SQLite + assets and plugin_web bundles.";
            };
            errorReportUrl = mkOption { type = types.nullOr types.str; default = null; };
            plugins = mkOption {
              default = [ ];
              description = "Plugins to register and run alongside the main server.";
              type = types.listOf (types.submodule ({ ... }: {
                options = {
                  name = mkOption {
                    type = types.str;
                    description = "Must match the plugin's `plugin.name` (e.g. timeline_plugin_steam).";
                  };
                  package = mkOption {
                    type = types.package;
                    description = "Built plugin package (from <plugin>.lib output): provides bin/<name>_server and share/web/.";
                  };
                  port = mkOption { type = types.port; };
                  token = mkOption {
                    type = types.str;
                    description = "Shared bearer token (also sent by the main server).";
                  };
                  displayName = mkOption { type = types.nullOr types.str; default = null; };
                  settings = mkOption {
                    type = tomlFormat.type;
                    default = { };
                    description = "Plugin-specific config; rendered as the [config] section of its config.toml.";
                  };
                };
              }));
            };
          };

          config = lib.mkIf cfg.enable {
            users.groups.${cfg.group} = { };
            users.users.${cfg.user} = { group = cfg.group; isSystemUser = true; };

            systemd.tmpfiles.rules = [
              "d ${cfg.dataDir} 0750 ${cfg.user} ${cfg.group} -"
              "d ${cfg.dataDir}/plugin_web 0750 ${cfg.user} ${cfg.group} -"
            ] ++ map (p:
              "L+ ${cfg.dataDir}/plugin_web/${p.name} - - - - ${p.package}/share/web"
            ) cfg.plugins;

            # Main server unit + per-plugin units, merged into one attrset so
            # there's a single `systemd.services` assignment (no eval collision).
            systemd.services = {
              timeline = {
                description = "Timeline main server";
                wantedBy = [ "multi-user.target" ];
                after = [ "network.target" ] ++ map (p: "timeline-plugin-${p.name}.service") cfg.plugins;
                serviceConfig = {
                  Type = "simple";
                  User = cfg.user;
                  Group = cfg.group;
                  WorkingDirectory = "${cfg.dataDir}/server";
                  ExecStartPre = [
                    "${pkgs.coreutils}/bin/mkdir -p ${cfg.dataDir}/server ${cfg.dataDir}/frontend"
                    "${pkgs.coreutils}/bin/ln -sfn ${cfg.frontendDist} ${cfg.dataDir}/frontend/dist"
                    "${pkgs.coreutils}/bin/cp -f ${serverConfigToml} ${cfg.dataDir}/server/config.toml"
                  ];
                  ExecStart = "${cfg.package}/bin/server";
                  Restart = "always";
                };
              };
            } // lib.listToAttrs (map (p: {
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
                    "${pkgs.coreutils}/bin/cp -f ${pluginConfig p} ${cfg.dataDir}/plugins/${p.name}/config.toml"
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
