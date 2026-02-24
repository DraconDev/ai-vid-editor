{
  description = "AI Video Editor - Automated video processing tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in
      {
        devShells.default = pkgs.mkShell {
          name = "ai-vid-editor-dev";
          
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            rust-analyzer
            rustfmt
            clippy
            
            # Build dependencies
            pkg-config
            openssl
            
            # GUI dependencies (for egui/wgpu)
            libGL
            libxkbcommon
            wayland
            
            # X11 support
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            
            # Video processing
            ffmpeg-full
            
            # File dialogs
            gtk3
          ];
          
          # Library path for dynamic linking
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.libGL
            pkgs.xorg.libX11
            pkgs.wayland
          ];
          
          shellHook = ''
            echo "AI Video Editor development environment"
            echo ""
            echo "Commands:"
            echo "  cargo build                         - Build CLI only"
            echo "  cargo build --features gui          - Build with GUI"
            echo "  cargo run --features gui --bin ai-vid-editor-gui  - Run GUI"
            echo ""
          '';
        };
      }
    );
}
