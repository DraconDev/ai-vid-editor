# NixOS development shell for ai-vid-editor
# Run with: nix-shell
# Or with flakes: nix develop

{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "ai-vid-editor-dev";
  
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
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
    
    # Video processing (ffmpeg)
    ffmpeg-full
    
    # For file dialogs
    gtk3
  ];
  
  # Library path for dynamic linking
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.libGL
    pkgs.xorg.libX11
    pkgs.wayland
  ];
  
  # Vulkan support for wgpu
  VK_ICD_FILENAMES = "${pkgs.vulkan-loader}/share/vulkan/icd.d/intel_icd.x86_64.json:${pkgs.vulkan-loader}/share/vulkan/icd.d/radeon_icd.x86_64.json:${pkgs.vulkan-loader}/share/vulkan/icd.d/nvidia_icd.json";
  
  shellHook = ''
    echo "AI Video Editor development environment"
    echo ""
    echo "Commands:"
    echo "  cargo build                    - Build CLI only"
    echo "  cargo build --features gui     - Build with GUI"
    echo "  cargo run --features gui --bin ai-vid-editor-gui  - Run GUI"
    echo ""
  '';
}
