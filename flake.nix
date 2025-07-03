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
            pkgs.wasm-pack
            pkgs.pkg-config
            pkgs.openssl
            pkgs.nodejs
          ];

          RUSTFLAGS = "--cfg=web_sys_unstable_apis";
          CARGO_TARGET_DIR = "target";

          shellHook = ''
            echo "Welcome to Ayaka dev shell"
          '';
        };
      }
    );
}

