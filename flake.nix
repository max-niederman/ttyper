{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    nixpkgs.url = "nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk.url = "github:nmattia/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rust = with fenix.packages.${system}; stable;
        naersk-lib = naersk.lib.${system}.override {
          inherit (rust) rustc cargo;
        };
      in
      rec {
        # `nix build`
        packages.ttyper = naersk-lib.buildPackage {
          pname = "ttyper";
          root = ./.;
        };
        defaultPackage = packages.ttyper;

        # `nix run`
        apps.ttyper = flake-utils.lib.mkApp {
          drv = packages.ttyper;
        };
        defaultApp = apps.ttyper;

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            (rust.withComponents [ "rustc" "cargo" "rust-src" "rustfmt" "clippy" ])
            rust-analyzer
          ];
        };
      }
    );
}
