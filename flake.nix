{
  description = "Plugin collection for anyrun.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-parts,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        {
          self',
          system,
          config,
          lib,
          pkgs,
          ...
        }:
        let
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              # includes already:
              # rustc
              # cargo
              # rust-std
              # rust-docs
              # rustfmt-preview
              # clippy-preview
              "rust-analyzer"
              "rust-src"
            ];
          };
          # Get all workspace members dynamically
          workspaceToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          members = workspaceToml.workspace.members;
          version = workspaceToml.workspace.package.version;
          mkPlugin =
            name:
            pkgs.callPackage ./nix/plugin.nix {
              inherit inputs;
              inherit name;
            };
          memberPackages = builtins.listToAttrs (
            map (member: {
              name = member;
              value = mkPlugin member;
            }) members
          );
        in
        {
          # overlay
          # https://flake.parts/overlays.html#consuming-an-overlay
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };
          packages = memberPackages // {
            # Bundle all workspace members into a single package
            anyrun-plugins = pkgs.symlinkJoin {
              name = "anyrun-plugins-${version}";
              paths = builtins.attrValues memberPackages;
              meta = {
                description = "A plugin collection for anyrun.";
                license = [ pkgs.lib.licenses.asl20 ];
                platforms = pkgs.lib.platforms.unix;
              };
            };

            default = self'.packages.anyrun-plugins;
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = builtins.attrValues memberPackages;
            packages = [
              rustToolchain
              pkgs.taplo
            ];
            env = {
              RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            };
            shellHook = ''
              echo "Rust development shell ready! 🦀 $(rustc --version)"
            '';
          };
        };
    };
}
