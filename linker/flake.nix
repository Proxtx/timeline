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
      in 
      {
        packages.default = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
        };
      }
    );
}