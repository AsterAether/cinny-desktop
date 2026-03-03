{
  description = "Cinny Desktop - Tauri development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Pin a specific Rust version for reproducible builds
        rustVersion = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        # WebKitGTK and related dependencies with proper glibc compatibility
        webkitDeps = with pkgs; [
          webkitgtk_4_1
          gtk3
          cairo
          gdk-pixbuf
          glib
          dbus
          openssl
          librsvg
          libsoup_3
        ];

        gstDeps = with pkgs; [
          gst_all_1.gstreamer
          gst_all_1.gst-plugins-base
          gst_all_1.gst-plugins-good
          gst_all_1.gst-plugins-bad
          gst_all_1.gst-plugins-ugly
          gst_all_1.gst-libav
          # Audio system dependencies
          alsa-lib
          pulseaudio
          pipewire
        ];

        # System libraries needed for Tauri
        systemDeps = with pkgs; [
          pkg-config
          wrapGAppsHook3
          glib-networking
          shared-mime-info
          gsettings-desktop-schemas
        ];

        # Build tools
        buildDeps = with pkgs; [
          nodejs_20
          git
          curl
          wget
          unzip
        ];

        # Nixpkgs with unfree packages allowed (needed for Android SDK)
        pkgsUnfree = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
          config.android_sdk.accept_license = true;
        };

        # Android SDK composition
        androidComposition = pkgsUnfree.androidenv.composeAndroidPackages {
          platformVersions = [ "34" "36" ];
          buildToolsVersions = [ "34.0.0" "35.0.0" ];
          includeNDK = true;
          ndkVersions = [ "25.2.9519653" ];
          includeSources = false;
          includeSystemImages = true;
          systemImageTypes = [ "google_apis" ];
          abiVersions = [ "x86_64" "arm64-v8a" ];
          includeEmulator = true;
          extraLicenses = [
            "android-sdk-license"
            "android-sdk-preview-license"
            "android-googletv-license"
            "android-sdk-arm-dbt-license"
            "google-gdk-license"
            "intel-android-extra-license"
            "intel-android-sysimage-license"
            "mips-android-sysimage-license"
          ];
        };

        androidSdk = androidComposition.androidsdk;
        androidEmulator = androidComposition.emulator;
        jdk = pkgs.jdk17;

        # Rust toolchain with Android cross-compilation targets
        rustVersionAndroid = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
          targets = [
            "aarch64-linux-android"
            "armv7-linux-androideabi"
            "i686-linux-android"
            "x86_64-linux-android"
          ];
        };

      in
      {
        devShells.default = pkgs.mkShell rec {
          buildInputs = webkitDeps ++ gstDeps ++ gstDeps ++ systemDeps ++ buildDeps ++ [ rustVersion ];

          # Environment variables to ensure proper library linking
          shellHook = ''
            export RUST_SRC_PATH="${rustVersion}/lib/rustlib/src/rust/library"
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.libsoup_3.dev}/lib/pkgconfig:${pkgs.webkitgtk_4_1.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath webkitDeps}:$LD_LIBRARY_PATH"
            export WEBKIT_DISABLE_COMPOSITING_MODE=1
            export GST_PLUGIN_SYSTEM_PATH_1_0="${pkgs.lib.makeSearchPathOutput "lib" "lib/gstreamer-1.0" (with pkgs; [
              gst_all_1.gstreamer
              gst_all_1.gst-plugins-base
              gst_all_1.gst-plugins-good
              gst_all_1.gst-plugins-bad
              gst_all_1.gst-plugins-ugly
            ])}"

            # Ensure consistent glibc version
            export NIX_ENFORCE_PURITY=0
            export RUSTFLAGS="-C link-arg=-Wl,--allow-multiple-definition"

            echo "🦀 Rust development environment for Cinny Desktop"
            echo "Node version: $(node --version)"
            echo "NPM version: $(npm --version)"
            echo "Rust version: $(rustc --version)"
            echo "WebKitGTK version: ${pkgs.webkitgtk_4_1.version}"
            echo ""
            echo "Ready to build Cinny Desktop!"
            echo "Run for complete build: npm run tauri build"
            echo "Run for .deb testing:   dpkg -x src-tauri/target/release/bundle/deb/Cinny_4.10.2_amd64.deb /tmp/cinny-test"
            echo "Run for .deb build:     npm run tauri build -- --bundles deb"
          '';

          # GApps wrapper for proper GTK theme integration
          nativeBuildInputs = with pkgs; [
            wrapGAppsHook3
            pkg-config
          ];

          # Additional environment for GTK applications
          XDG_DATA_DIRS = pkgs.lib.concatStringsSep ":" [
            "${pkgs.gsettings-desktop-schemas}/share"
            "${pkgs.gtk3}/share"
            "$XDG_DATA_DIRS"
          ];
        };

        devShells.android = pkgs.mkShell rec {
          buildInputs = systemDeps ++ buildDeps ++ [
            rustVersionAndroid
            jdk
            androidSdk
            pkgs.gradle
          ];

          ANDROID_HOME = "${androidSdk}/libexec/android-sdk";
          NDK_HOME = "${ANDROID_HOME}/ndk/25.2.9519653";
          JAVA_HOME = "${jdk}";

          shellHook = ''
            export RUST_SRC_PATH="${rustVersionAndroid}/lib/rustlib/src/rust/library"
            export ANDROID_HOME="${ANDROID_HOME}"
            export NDK_HOME="${NDK_HOME}"
            export JAVA_HOME="${JAVA_HOME}"
            export GRADLE_OPTS="-Dorg.gradle.project.android.aapt2FromMavenOverride=${ANDROID_HOME}/build-tools/34.0.0/aapt2"
            export NIX_ENFORCE_PURITY=0
            export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"
            export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi21-clang"
            export CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android21-clang"
            export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android21-clang"
            export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=0x10000 -C link-arg=-Wl,-z,common-page-size=0x10000"
            export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=0x10000 -C link-arg=-Wl,-z,common-page-size=0x10000"
            export CARGO_TARGET_I686_LINUX_ANDROID_RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=0x10000 -C link-arg=-Wl,-z,common-page-size=0x10000"
            export CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=0x10000 -C link-arg=-Wl,-z,common-page-size=0x10000"

            echo "Android development environment for Cinny Desktop"
            echo "Node version: $(node --version)"
            echo "Rust version: $(rustc --version)"
            echo "Java version: $(java --version 2>&1 | head -1)"
            echo "ANDROID_HOME: $ANDROID_HOME"
            echo "NDK_HOME: $NDK_HOME"
            echo ""
            echo "Initialize:    npx tauri android init"
            echo "Run on device: npx tauri android dev"
            echo "Build APK:     npx tauri android build"
          '';

          nativeBuildInputs = with pkgs; [ pkg-config ];
        };

        # Optional: Package definition for the built application
        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cinny-desktop";
          version = "4.10.2";

          src = ./.;

          cargoLock = {
            lockFile = ./src-tauri/Cargo.lock;
          };

          buildInputs = webkitDeps ++ gstDeps ++ systemDeps;
          nativeBuildInputs = with pkgs; [ pkg-config wrapGAppsHook3 nodejs_20 ];

          # Build the frontend first
          preBuild = ''
            cd cinny
            npm ci
            npm run build
            cd ..
            cp config.json cinny/
          '';

          buildAndTestSubdir = "src-tauri";

          meta = with pkgs.lib; {
            description = "Yet another matrix client";
            homepage = "https://github.com/cinnyapp/cinny-desktop";
            license = licenses.agpl3Only;
            maintainers = [ ];
            platforms = platforms.linux;
          };
        };
      }
    );
}
