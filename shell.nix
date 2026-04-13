{
  mkShell,
  rustPlatform,
  rust-bin,
  mold,
  cargo-flamegraph,
  cargo-msrv,
  cargo-license,
  cargo-deny,
  pkg-config,
  udev,
  ...
}: let
  nightlyToolchain = rust-bin.selectLatestNightlyWith (toolchain:
    toolchain.default.override {
      extensions = [
        "rust-src"
        "rust-analyzer-preview"
        "clippy-preview"
        "rustfmt-preview"
      ];
      targets = [
        "x86_64-unknown-linux-musl"
        "aarch64-unknown-linux-musl"
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
      ];
    });
in
  mkShell {
    strictDeps = true;

    packages = [
      cargo-flamegraph
      cargo-msrv
      cargo-license
      cargo-deny
      mold
      nightlyToolchain
    ];
    nativeBuildInputs = [
      pkg-config
    ];
    buildInputs = [
      udev.dev
    ];

    env = {
      RUST_SRC_PATH = rustPlatform.rustLibSrc;
      CARGO_TERM_COLOR = "always";
      RUST_BACKTRACE = "1";
    };
  }
