{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      nixpkgs,
      flake-parts,
      rust-overlay,
      ...
    }:

    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      imports = with inputs; [
        git-hooks.flakeModule
        treefmt-nix.flakeModule
      ];

      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        let
          toolchain = pkgs.rust-bin.stable.latest.default;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        {
          _module.args.pkgs = import nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          packages.default = rustPlatform.buildRustPackage {
            pname = "game-of-life";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.pre-commit.devShell
            ];
            inherit (config.packages.default) nativeBuildInputs;
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
              taplo.enable = true;
            };

            settings.formatter = {
              taplo.options = [
                "fmt"
                "-o"
                "reorder_keys=true"
              ];
            };
          };

          pre-commit = {
            check.enable = true;
            settings = {
              hooks = {
                ripsecrets.enable = true;
                typos.enable = true;
                treefmt.enable = true;
                cargo-check.enable = true;
                clippy = {
                  enable = true;
                  packageOverrides.cargo = toolchain;
                  packageOverrides.clippy = toolchain;
                };
              };
            };
          };
        };
    };
}
