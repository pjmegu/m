{
  description = "Project Megu Standard Development Environment on Nix";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.fenix;
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            (rust.combine [
              rust.default.toolchain
              rust.targets.wasm32-unknown-unknown.latest.toolchain
            ])
            pkgs.cargo-nextest
            pkgs.wasm-tools
          ];
        };
      });
}
