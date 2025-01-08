{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      crane,
    }:
    let
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs
          [
            "x86_64-linux"
            "aarch64-linux"
            "x86_64-darwin"
            "aarch64-darwin"
          ]
          (
            system:
            let
              pkgs = import nixpkgs {
                inherit system;
                overlays = [ (import rust-overlay) ];
              };
            in
            function pkgs
          );

      makePackage =
        pkgs:
        let
          rustToolchain = pkgs.rust-bin.stable.latest.default;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          files-to-prompt = pkgs.callPackage ./package.nix {
            inherit craneLib rustToolchain;
            system = pkgs.system;
          };
        in
        files-to-prompt;
    in
    {
      packages = forAllSystems (pkgs: {
        default = makePackage pkgs;
        files-to-prompt = makePackage pkgs;
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ (makePackage pkgs) ];
        };
      });

      overlays.default = final: prev: {
        files-to-prompt = self.packages.${prev.system}.files-to-prompt;
      };
    };
}
