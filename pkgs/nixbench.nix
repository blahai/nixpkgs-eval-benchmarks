{
  rustPlatform,
  makeBinaryWrapper,
  rust-bin,
  lib,
  pkg-config,
  udev,
  ...
}: let
  nightlyToolchain = rust-bin.selectLatestNightlyWith (toolchain:
    toolchain.default.override {
      extensions = ["rust-src"];
      targets = [
        "x86_64-unknown-linux-musl"
        "aarch64-unknown-linux-musl"
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
      ];
    });
  toml = (lib.importTOML ../nixbench/Cargo.toml).package;
in
  rustPlatform.buildRustPackage (finalAttrs: {
    name = "nixbench";
    inherit (toml) version;

    rustc = rust-bin.nightly.latest.default;
    cargo = rust-bin.nightly.latest.default;

    cargoLock = {
      lockFile = ../nixbench/Cargo.lock;
    };

    src = ../nixbench;

    nativeBuildInputs = [
      pkg-config
      nightlyToolchain
      makeBinaryWrapper
    ];

    buildInputs = [
      udev.dev
    ];

    meta = {
      inherit (toml) homepage description;
      mainProgram = "nixbench";
    };
  })
