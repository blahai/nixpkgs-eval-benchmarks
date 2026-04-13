{
  lib,
  stdenv,
  busybox,
  nix,
  inputs,
}:
stdenv.mkDerivation {
  pname = "attrpaths-superset";
  version = "1";

  src = null;
  dontUnpack = true;
  dontPatch = true;

  nativeBuildInputs = [
    (lib.getBin busybox)
    (lib.getBin nix)
  ];

  buildPhase = ''
    export NIX_STATE_DIR=$(mktemp -d)
    mkdir -p $out
    export GC_INITIAL_HEAP_SIZE=4g
    command time -f "Attribute eval done [%MKB max resident, %Es elapsed] %C" \
      nix-instantiate --eval --strict --json --show-trace \
        "${inputs.nixpkgs}/ci/eval/attrpaths.nix" \
        -A paths \
        -I "${inputs.nixpkgs}" \
        --argstr extraNixpkgsConfigJson {} \
        --option restrict-eval true \
        --option allow-import-from-derivation false \
        --option eval-system "${stdenv.hostPlatform.system}" > $out/paths.json
  '';

  dontInstall = true;
  dontFixup = true;
}
