{
  description = "Ayaka build environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
        ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.cargo-tauri
            pkgs.glib
            pkgs.gobject-introspection
            pkgs.wasm-pack
            pkgs.pango
            pkgs.atkmm
            pkgs.webkitgtk_4_1
            pkgs.pkg-config
            pkgs.gtk3.out
            pkgs.mesa
            pkgs.gsettings-desktop-schemas
            pkgs.openssl
            pkgs.nodejs
          ];

          RUSTFLAGS = "--cfg=web_sys_unstable_apis";
          CARGO_TARGET_DIR = "target";

          shellHook = ''
            export LIBGL_ALWAYS_SOFTWARE=1
            export GTK_PATH="${pkgs.gtk3}/lib/gtk-3.0"
            echo "Welcome to Ayaka dev shell"
          '';
        };
      }
    );
}

