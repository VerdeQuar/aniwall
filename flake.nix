{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          # rustVersion = "1.70.0";
          rustChannel = "nightly";
          packageFun = import ./Cargo.nix;
        };

      in rec {
        packages = {
          aniwall = (rustPkgs.workspace.aniwall {});
          default = packages.aniwall;
        };
        devShells.default = pkgs.mkShell {
          name = "aniwall";
          packages = with pkgs; [
            openssl
            pkg-config
          ];
        };
      }
    );
}
