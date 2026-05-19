{
  description = "AttackTower — Rust/Bevy project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };

        # System libraries required by Bevy at build time and run time.
        buildInputs = with pkgs; [
          # Audio
          alsa-lib
          # Input devices (gamepads, etc.)
          udev
          # Windowing
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          # Graphics
          vulkan-loader
          vulkan-tools
          vulkan-headers
          vulkan-validation-layers
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          # Faster linker, recommended by Bevy for incremental builds.
          clang
          mold
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          packages = [
            rustToolchain
            pkgs.cargo-watch
          ];

          # Bevy loads Vulkan and other libs dynamically at runtime; they must be
          # discoverable via LD_LIBRARY_PATH inside the dev shell.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

          # Use mold as the linker for faster Rust builds.
          RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
