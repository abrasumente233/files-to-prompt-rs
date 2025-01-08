{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
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
      in
      with pkgs;
      {
        overlays = final: prev: {
          files-to-prompt = files-to-prompt;
        };

        packages.default = files-to-prompt;
        packages.files-to-prompt = files-to-prompt;

        devShells.default = mkShell {
          inputsFrom = [ files-to-prompt ];
        };
      }
    );
}
