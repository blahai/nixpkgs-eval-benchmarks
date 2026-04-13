{
  allNixVersions,
  legacyOutputs,
  lib,
  stdenv,
  legacy ? false,
}: let
  nixStorePaths =
    builtins.map (drv: toString drv) (builtins.attrValues allNixVersions);
  legacyStorePaths =
    builtins.map (drv: toString drv) (builtins.attrValues legacyOutputs);
in
  stdenv.mkDerivation (finalAttrs: {
    pname = "nix-versions-json";
    version = "1";

    src = null;
    dontUnpack = true;
    dontPatch = true;
    dontInstall = true;

    buildInputs =
      [
        (builtins.attrValues allNixVersions)
      ]
      ++ lib.optionals legacy
      [
        (builtins.attrValues legacyOutputs)
      ];

    name = "nix-store-paths";

    buildPhase =
      ''
        mkdir -p $out
        cat > $out/paths.json <<'EOF'
        ${lib.generators.toJSON {} nixStorePaths}
        EOF
      ''
      + lib.optionalString legacy ''
        cat > $out/paths-legacy.json <<'EOF'
        ${lib.generators.toJSON {} (nixStorePaths ++ legacyStorePaths)}
        EOF
      '';
  })
