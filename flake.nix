{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    # rust
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        files-to-prompt = pkgs.callPackage ./package.nix { inherit craneLib rustToolchain system; };

        overlay = final: prev: {
          files-to-prompt = files-to-prompt;
        };
      in
      with pkgs;
      {
        overlays.default = overlay;
        overlays.files-to-prompt = overlay;

        packages.default = files-to-prompt;
        packages.files-to-prompt = files-to-prompt;

        devShells.default = mkShell {
          inputsFrom = [ files-to-prompt ];
        };
      }
    );
}
