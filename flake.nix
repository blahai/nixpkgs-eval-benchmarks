{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    old-nixpkgs.url = "github:nixos/nixpkgs/48d12d5e70ee91fe8481378e540433a7303dbf6a";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nix-src = {
      url = "github:nixos/nix";
    };
    nix-227-src = {
      url = "github:nixos/nix/2.27-maintenance";
      # needed or this explodes
      inputs.nixpkgs.follows = "old-nixpkgs";
    };
    nix-226-src = {
      url = "github:nixos/nix/2.26-maintenance";
    };
    nix-225-src = {
      url = "github:nixos/nix/2.25-maintenance";
    };
    nix-224-src = {
      url = "github:nixos/nix/2.24-maintenance";
    };
    nix-223-src = {
      url = "github:nixos/nix/2.23-maintenance";
    };
    nix-222-src = {
      url = "github:nixos/nix/2.22-maintenance";
    };
    nix-221-src = {
      url = "github:nixos/nix/2.21-maintenance";
    };
    nix-220-src = {
      url = "github:nixos/nix/2.20-maintenance";
    };
    nix-219-src = {
      url = "github:nixos/nix/2.19-maintenance";
    };
    nix-218-src = {
      url = "github:nixos/nix/2.18-maintenance";
    };

    lix-src = {
      url = "https://git.lix.systems/lix-project/lix/archive/main.tar.gz";
    };
    lix-292-src = {
      url = "https://git.lix.systems/lix-project/lix/archive/2.92.3.tar.gz";
    };
    lix-291-src = {
      url = "https://git.lix.systems/lix-project/lix/archive/2.91.3.tar.gz";
    };
  };
  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    ...
  } @ inputs: let
    inherit (nixpkgs) lib;
    inherit (lib.attrsets) genAttrs;
    overlays = [(import rust-overlay)];

    systems = ["x86_64-linux" "aarch64-linux"];

    forAllSystems = fn:
      genAttrs systems (
        system:
          fn (
            import nixpkgs {
              inherit system overlays;
              config = {
                allowUnfree = true;
                allowAliases = false;
              };
            }
          )
      );
  in {
    devShells = forAllSystems (pkgs: {
      default = pkgs.callPackage ./shell.nix {};
    });

    apps = forAllSystems (pkgs: {
      nixbenchWrapped = {
        type = "app";
        program = let
          script = pkgs.writeShellApplication {
            name = "nixbench-wrapped";
            runtimeInputs = [
              self.packages.${pkgs.stdenv.hostPlatform.system}.nixbench
              self.packages.${pkgs.stdenv.hostPlatform.system}.nix-versions-json
              self.packages.${pkgs.stdenv.hostPlatform.system}.attrpathsSuperset
            ];
            text = ''
              exec ${self.packages.${pkgs.stdenv.hostPlatform.system}.nixbench}/bin/nixbench \
                --nix-paths ${self.packages.${pkgs.stdenv.hostPlatform.system}.nix-versions-json}/paths.json \
                --nixpkgs ${inputs.nixpkgs} \
                --attrpaths ${self.packages.${pkgs.stdenv.hostPlatform.system}.attrpathsSuperset}/paths.json \
                --chunk-size 15000 \
                "$@"
            '';
          };
        in
          pkgs.lib.getExe script;
      };

      nixbenchWrappedLegacy = {
        type = "app";
        program = let
          script = pkgs.writeShellApplication {
            name = "nixbench-wrapped-legacy";
            runtimeInputs = [
              self.packages.${pkgs.stdenv.hostPlatform.system}.nixbench
              self.packages.${pkgs.stdenv.hostPlatform.system}.nix-versions-json-legacy
              self.packages.${pkgs.stdenv.hostPlatform.system}.attrpathsSuperset
            ];
            text = ''
              exec ${self.packages.${pkgs.stdenv.hostPlatform.system}.nixbench}/bin/nixbench \
                --nix-paths ${self.packages.${pkgs.stdenv.hostPlatform.system}.nix-versions-json-legacy}/paths-legacy.json \
                --nixpkgs ${inputs.nixpkgs} \
                --attrpaths ${self.packages.${pkgs.stdenv.hostPlatform.system}.attrpathsSuperset}/paths.json \
                --chunk-size 15000 \
                "$@"
            '';
          };
        in
          pkgs.lib.getExe script;
      };
    });

    packages = forAllSystems (pkgs: let
      nixOutputs =
        builtins.mapAttrs
        (_: version: pkgs.nixVersions.${version})
        {
          # don't use nixpkgs git since we have the flake for master
          # nix-git = "git";
          nix-latest = "latest";
          nix-stable = "stable";
          nix-234 = "nix_2_34";
          nix-231 = "nix_2_31";
          nix-230 = "nix_2_30";
          nix-228 = "nix_2_28";
        };

      lixOutputs =
        builtins.mapAttrs
        (_: version: pkgs.lixPackageSets.${version}.lix)
        {
          # don't use nixpkgs git since we have the flake for master
          # lix-git = "git";
          lix-latest = "latest";
          lix-stable = "stable";
          lix-295 = "lix_2_95";
          lix-294 = "lix_2_94";
          lix-293 = "lix_2_93";
        };

      inputOutputs =
        builtins.mapAttrs
        (_: input: inputs.${input}.packages.${pkgs.stdenv.hostPlatform.system}.default)
        {
          nix-master = "nix-src";
          lix-main = "lix-src";
        };

      legacyOutputs =
        builtins.mapAttrs
        (_: input: inputs.${input}.packages.${pkgs.stdenv.hostPlatform.system}.default)
        {
          nix-227 = "nix-227-src";
          nix-226 = "nix-226-src";
          nix-225 = "nix-225-src";
          nix-224 = "nix-224-src";
          nix-223 = "nix-223-src";
          nix-222 = "nix-222-src";
          nix-221 = "nix-221-src";
          nix-220 = "nix-220-src";
          nix-219 = "nix-219-src";
          nix-218 = "nix-218-src";

          lix-292 = "lix-292-src";
          lix-291 = "lix-291-src";
        };

      allNixVersions = nixOutputs // lixOutputs // inputOutputs;
    in
      allNixVersions
      // legacyOutputs
      // {
        attrpathsSuperset = pkgs.callPackage ./pkgs/attrpaths.nix {inherit inputs;};
        nix-versions-json = pkgs.callPackage ./pkgs/nix-versions-json.nix {inherit allNixVersions legacyOutputs;};
        nix-versions-json-legacy = self.packages.${pkgs.stdenv.hostPlatform.system}.nix-versions-json.override {legacy = true;};
        nixbench = pkgs.callPackage ./pkgs/nixbench.nix {};
      });
  };
}
