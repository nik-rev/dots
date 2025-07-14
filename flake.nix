{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forEachSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          let
            overlays = [ (import rust-overlay) ];
            pkgs = import nixpkgs {
              inherit system overlays;
            };
            nativeBuildInputs = [
              (pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            ];
          in
          f {
            inherit system pkgs nativeBuildInputs;
          }
        );
    in
    {
      devShells = forEachSystem (
        { pkgs, nativeBuildInputs, ... }:
        {
          default = pkgs.mkShell {
            inherit nativeBuildInputs;
            buildInputs = [ pkgs.nixfmt-rfc-style ];
          };
        }
      );

      packages = forEachSystem (
        { pkgs, nativeBuildInputs, ... }:
        {
          default =
            let
              manifest = pkgs.lib.importTOML ./Cargo.toml;
            in
            pkgs.rustPlatform.buildRustPackage {
              pname = manifest.package.name;
              version = manifest.package.version;
              src = pkgs.lib.cleanSource ./.;
              cargoLock.lockFile = ./Cargo.lock;

              inherit nativeBuildInputs;
            };
        }
      );
    };
}
