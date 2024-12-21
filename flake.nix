{
  description = "Zellij Prime Hopper plugin devshell";

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
        inputs.git-hooks-nix.flakeModule
        inputs.treefmt-nix.flakeModule

        ./nix/cmd-fmt.nix
        ./nix/devshells.nix
        ./nix/package.nix
      ];

      systems = ["aarch64-darwin" "aarch64-linux" "x86_64-linux"];

      perSystem = {system, ...}: let
        isDarwin = system == "aarch64-darwin";
        nixpkgs =
          if isDarwin
          then inputs.nixpkgs-darwin
          else inputs.nixpkgs;
      in {
        _module.args.pkgs = import nixpkgs {
          inherit system;
          overlays = [inputs.rust-overlay.overlays.default];
        };
      };
    };

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    nixpkgs-darwin.url = "github:NixOS/nixpkgs/nixpkgs-24.11-darwin";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    # Pure and reproducible packaging of binary distributed rust toolchains.
    rust-overlay.url = "github:oxalica/rust-overlay";

    # We use flake parts to organize our configurations.
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs-unstable";
      inputs.nixpkgs-stable.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
