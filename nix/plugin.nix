{
  inputs,
  lib,
  pkgs,
  name,
  ...
}:
let
  workspaceCargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  cargoToml = builtins.fromTOML (builtins.readFile ../${name}/Cargo.toml);
  cargoLock = ../Cargo.lock;
  pname = cargoToml.package.name;
  version = workspaceCargoToml.workspace.package.version;
  rustPlatform = pkgs.makeRustPlatform {
    cargo = pkgs.rust-bin.stable.latest.minimal;
    rustc = pkgs.rust-bin.stable.latest.minimal;
  };
in
rustPlatform.buildRustPackage {
  inherit pname version;

  src = builtins.path {
    path = lib.sources.cleanSource inputs.self;
    name = "${pname}-${version}";
  };

  strictDeps = true;

  nativeBuildInputs = with pkgs; [
    pkg-config
    makeWrapper
  ];

  buildInputs = [
    pkgs.openssl
  ];
  doCheck = true;
  checkInputs = with pkgs; [
    cargo
    rustc
  ];

  copyLibs = true;
  cargoBuildFlags = [ "-p ${name}" ];
  buildAndTestSubdir = "${name}";

  CARGO_BUILD_INCREMENTAL = "false";
  RUST_BACKTRACE = "full";

  cargoLock = {
    lockFile = cargoLock;
    allowBuiltinFetchGit = true;
  };

  # Disable cargo-auditable until https://github.com/rust-secure-code/cargo-auditable/issues/124 is fixed
  auditable = false;

  meta = {
    homepage = "https://github.com/thinkgos/anyrun-plugins/${pname}";
    description = "The ${name} plugin for Anyrun";
    license = [ lib.licenses.asl20 ];
    maintainers = with lib.maintainers; [
      thinkgos
    ];
  };
}
