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

    # Node.js for frontend
    nodejs_22
  ];

  shellHook = ''
    echo "ordiseq_app dev shell"
    echo "Run: cargo tauri dev"
  '';
}
