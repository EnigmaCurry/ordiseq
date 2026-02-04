# Nix shell for ordiseq_app (Tauri v2 + Svelte)
# Enter with: nix-shell
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain (use rustup from parent project)
    pkg-config

    # Tauri dependencies for Linux
    webkitgtk_4_1
    gtk3
    glib
    cairo
    pango
    gdk-pixbuf
    libsoup_3
    openssl

    # Additional Tauri v2 deps
    librsvg
    alsa-lib

    # Libraries needed by cargo-tauri binary
    bzip2
    zlib

    # Graphics libraries
    mesa

    # Node.js for frontend
    nodejs_22

    # Windows cross-compilation
    llvmPackages.clang
    llvmPackages.lld
  ];

  shellHook = ''
    # Add library paths for cargo-tauri binary compatibility
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
      pkgs.bzip2
      pkgs.zlib
      pkgs.openssl
      pkgs.glib
      pkgs.gdk-pixbuf
      pkgs.cairo
      pkgs.pango
      pkgs.gtk3
      pkgs.webkitgtk_4_1
      pkgs.libsoup_3
      pkgs.alsa-lib
      pkgs.mesa
    ]}:$LD_LIBRARY_PATH"

    # WebKitGTK workarounds for EGL issues
    export WEBKIT_DISABLE_DMABUF_RENDERER=1
    export WEBKIT_DISABLE_COMPOSITING_MODE=1
    export LIBGL_ALWAYS_SOFTWARE=1

    echo "ordiseq_app dev shell"
    echo "Run: cargo tauri dev"
  '';
}
