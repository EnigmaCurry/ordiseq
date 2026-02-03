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

    # Node.js for frontend
    nodejs_22
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
    ]}:$LD_LIBRARY_PATH"

    echo "ordiseq_app dev shell"
    echo "Run: cargo tauri dev"
  '';
}
