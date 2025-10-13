{
  description = "agendrr";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      pkgsForSystem =
        system:
        (import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        });
    in
    {
      packages = forAllSystems (
        system:
        let
          inherit (pkgsForSystem system)
            lib
            rustPlatform
            ;

          cargoToml = lib.trivial.importTOML ./Cargo.toml;
          version = cargoToml.package.version;
        in
        rec {
          default = agendrr;

          agendrr = rustPlatform.buildRustPackage {
            pname = "agendrr";
            version = version;
            src = lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "Generate markdown summaries from your Google Calendar";
              homepage = "https://github.com/jnsgruk/agendrr";
              license = lib.licenses.asl20;
              mainProgram = "agendrr";
              platforms = lib.platforms.unix;
              maintainers = with lib.maintainers; [ jnsgruk ];
            };
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsForSystem system;
          rust = pkgs.rust-bin.beta.latest.default.override {
            extensions = [ "rust-src" ];
          };
        in
        {
          default = pkgs.mkShell {
            name = "agendrr";

            NIX_CONFIG = "experimental-features = nix-command flakes";
            RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";

            inputsFrom = [ self.packages.${system}.agendrr ];
            buildInputs = with pkgs; [
              rust
              nil
              nixfmt-rfc-style
            ];
          };
        }
      );
    };
}
