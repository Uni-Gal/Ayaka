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
          config = {
            # Allow android sdk
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        };
        
        # Configure Android packages with NDK
        androidComposition = pkgs.androidenv.composeAndroidPackages {
          includeNDK = true;
          ndkVersions = ["25.1.8937393"];
        };

        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Linux Target
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
            
            # Android
            #pkgs.android-tools
            #androidComposition.androidsdk
          ];

          RUSTFLAGS = "--cfg=web_sys_unstable_apis";
          CARGO_TARGET_DIR = "target";

          shellHook = ''
            export LIBGL_ALWAYS_SOFTWARE=1
            export GTK_PATH="${pkgs.gtk3}/lib/gtk-3.0"
            #export ANDROID_HOME=${androidComposition.androidsdk}/libexec/android-sdk
            #export ANDROID_SDK_ROOT=$ANDROID_HOME
            #export ANDROID_NDK_ROOT=$ANDROID_HOME/ndk-bundle
            #export NDK_HOME=$ANDROID_NDK_ROOT
            #export PATH=$ANDROID_HOME/platform-tools:$ANDROID_NDK_ROOT:$PATH
            #echo "Android SDK configured at $ANDROID_HOME"
            #echo "Android NDK configured at $ANDROID_NDK_ROOT"

            echo "Welcome to Ayaka dev shell!"
          '';
        };
      }
    );
}
