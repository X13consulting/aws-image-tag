{
  description = "A devShell example";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        entrypoint = pkgs.writeScriptBin "entrypoint" ''
          #!${pkgs.stdenv.shell}
          mkdir /usr
          ln -s bin /usr/bin
          exec aws-image-tag
        '';
        rustVersion = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };
        myRustBuild = rustPlatform.buildRustPackage {
          pname = "aws-image-tag";
          version = rustVersion;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
          pathsToLink = [ "/bin" ];
        };
        dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = myRustBuild.pname;
          tag = "latest";
          contents = [
            entrypoint
            pkgs.coreutils
            pkgs.bash
            myRustBuild
          ];
          config = {
            Cmd = [ entrypoint ];
          };
        };
      in
      with pkgs;
      {
        packages = {
         rustPackage = myRustBuild;
         docker = dockerImage;
        };
        defaultPackage = dockerImage;
        devShells.default = mkShell {
          buildInputs = [
            openssl
            pkg-config
            eza
            fd
            rust-bin.stable.latest.default
            rust-analyzer
          ];
          shellHook = ''
            alias ls=eza
            alias find=fd
          '';
        };
      }
    );
}

