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
          libx11
          libxcursor
          libxi
          libxrandr
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

        # `nix run .#release [-- "tag message"]` — derives the tag from
        # Cargo.toml and pushes it. The GitHub Actions workflow takes over
        # from there (build + publish per-platform artifacts).
        releaseApp = pkgs.writeShellApplication {
          name = "release";
          runtimeInputs = with pkgs; [ git gnugrep coreutils ];
          text = ''
            cd "$(git rev-parse --show-toplevel)"
            version=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
            if [ -z "$version" ]; then
              echo "error: could not read version from Cargo.toml" >&2
              exit 1
            fi
            tag="v$version"
            message="''${1:-Release $tag}"
            if ! git diff-index --quiet HEAD --; then
              echo "error: working tree has uncommitted changes — commit or stash first" >&2
              exit 1
            fi
            if git rev-parse "$tag" >/dev/null 2>&1; then
              echo "error: tag $tag already exists locally" >&2
              exit 1
            fi
            if git ls-remote --tags --exit-code origin "refs/tags/$tag" >/dev/null 2>&1; then
              echo "error: tag $tag already exists on origin" >&2
              exit 1
            fi
            echo "Tagging $tag on $(git rev-parse --short HEAD)..."
            git tag -a "$tag" -m "$message"
            git push origin "$tag"
            echo "Done — the release workflow should now be running for $tag."
          '';
        };
      in
      {
        apps.release = flake-utils.lib.mkApp { drv = releaseApp; };

        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          packages = [
            rustToolchain
            pkgs.cargo-watch
            # Performance overlay: prefix the launch command with `mangohud`.
            pkgs.mangohud
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
