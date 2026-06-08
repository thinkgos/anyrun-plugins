# anyrun-plugins

A collection of [anyrun](https://github.com/anyrun-org/anyrun) plugins written in Rust.

[anyrun](https://github.com/anyrun-org/anyrun) is a Wayland-native, fuzzy-search application launcher. This repository hosts plugins that extend it with additional search providers and command runners.

## Plugins

| Plugin        | Description                                                       |
|---------------|-------------------------------------------------------------------|
| `ssh-pattern` | Fuzzy-search and connect to hosts defined in your SSH config files,<br> more information in [`ssh-pattern/README.md`](ssh-pattern/README.md). |
| `menu-bar`    | Two-level hierarchical menu bar for launching commands,<br> more information in [`menu-bar/README.md`](menu-bar/README.md). |

## Repository Layout

```sh
anyrun-plugins/
├── Cargo.toml            # Workspace manifest
├── flake.nix             # Nix flake (dev shell + per-plugin packages)
├── nix/
│   └── plugin.nix        # Reusable builder for each plugin
├── menu-bar/             # Hierarchical menu bar plugin
└── ssh-pattern/          # SSH search plugin
```

This is a Cargo workspace: each plugin lives in its own crate and is built as a
`cdylib` using `anyrun-plugin` and `abi_stable`.

## Requirements

- **Rust** 1.96 or newer (edition 2024)
- **Cargo**
- An [anyrun](https://github.com/anyrun-org/anyrun) install for running the plugins
- Optional: **Nix** with flakes enabled, for the provided dev shell and package builds

## Installation

### From a single plugin (recommended)

Each plugin is an independent crate. Build only what you need:

```sh
cargo build --release -p ssh-pattern
```

The resulting shared library lives in `target/release/libssh_pattern.so` and can
be loaded by anyrun through its standard config.

### Using Nix

The flake exposes a dev shell and a packaged `anyrun-plugins` bundle:

```sh
# input
inputs = {
  nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  
  anyrun-plugins = {
    url = "github:thinkgos/anyrun-plugins";
    inputs.nixpkgs.follows = "nixpkgs";
  };
}
```

Per-system plugin packages are also available (e.g. `nix build .#ssh-pattern`).

## Plugin Configuration

Each plugin reads its config from `<anyrun-config-dir>/<plugin-name>.ron`. See
each plugin's `examples/` directory for a starter config:

```sh
cp ssh-pattern/examples/ssh-pattern.ron ~/.config/anyrun/ssh-pattern.ron
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
