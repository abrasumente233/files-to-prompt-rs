{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs@{
      flake-parts,
      nixpkgs,
      crane,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.devenv.flakeModule
      ];
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          lib,
          pkgs,
          system,
          ...
        }:

        let
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };
          packages.default = pkgs.callPackage ./package.nix { inherit craneLib rustToolchain; };
          devenv.shells.default =
            let
              isDarwin = pkgs.lib.strings.hasSuffix "-darwin" system;
            in
            {
              packages =
                [ rustToolchain ]
                ++ lib.optionals isDarwin [
                  pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
                  pkgs.libiconv
                ];
            };
        };
    };
}
