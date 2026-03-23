{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    inherit (nixpkgs) lib;
    inherit (lib.attrsets) genAttrs;

    systems = ["x86_64-linux" "aarch64-linux"];

    forAllSystems = fn:
      genAttrs systems (
        system:
          fn (
            import nixpkgs {
              inherit system;
              config = {
                allowUnfree = true;
                allowAliases = false;
              };
            }
          )
      );

    nixVersionMap = {
      nix-git = "git";
      nix-latest = "latest";
      nix-stable = "stable";
      nix-234 = "nix_2_34";
      nix-233 = "nix_2_33";
      nix-232 = "nix_2_32";
      nix-231 = "nix_2_31";
      nix-230 = "nix_2_30";
      nix-228 = "nix_2_28";
    };

    lixVersionMap = {
      lix-git = "git";
      lix-latest = "latest";
      lix-stable = "stable";
      lix-294 = "lix_2_94";
      lix-293 = "lix_2_93";
    };
  in {
    packages = forAllSystems (
      pkgs: let
        mkPackageWithNix = _: nixVersionName: let
          eval =
            (pkgs.callPackage "${nixpkgs}/ci/eval/default.nix" {
              nix = pkgs.nixVersions.${nixVersionName};
            }) {
              chunkSize = 15000;
            };
          inherit (eval) singleSystem;
        in
          singleSystem {
            evalSystem = pkgs.stdenv.hostPlatform.system;
          };

        mkPackageWithLix = _: lixVersionName: let
          eval =
            (pkgs.callPackage "${nixpkgs}/ci/eval/default.nix" {
              nix = pkgs.lixPackageSets.${lixVersionName}.lix;
            }) {
              chunkSize = 15000;
            };
          inherit (eval) singleSystem;
        in
          singleSystem {
            evalSystem = pkgs.stdenv.hostPlatform.system;
          };
      in
        (genAttrs (builtins.attrNames nixVersionMap) (
          name:
            mkPackageWithNix name nixVersionMap.${name}
        ))
        // (genAttrs (builtins.attrNames lixVersionMap) (
          name:
            mkPackageWithLix name lixVersionMap.${name}
        ))
    );

    apps = forAllSystems (pkgs: {
      build-all = {
        type = "app";
        program = "${pkgs.writeShellScript "build-all" ''
          set -euo pipefail

          packages=(${lib.concatStringsSep " " (builtins.attrNames self.packages.${pkgs.stdenv.hostPlatform.system})})

          for package in "''${packages[@]}"; do
            echo "Building $package..."
            nix build ".#$package" --out-link "result-$package"
            echo "Built $package, eval took: $(cat result-$package/*/total-time) seconds"
          done
        ''}";
      };
    });
  };
}
