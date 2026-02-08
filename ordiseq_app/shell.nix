# Nix shell for ordiseq_app (Tauri v2 + Svelte)
# Enter with: nix-shell
{ pkgs ? import <nixpkgs> {} }:

let
  # Create an ALSA config that uses PipeWire
  alsaConf = pkgs.writeText "asound.conf" ''
    pcm.!default {
      type pipewire
    }
    ctl.!default {
      type pipewire
    }
  '';
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain (use rustup from parent project)
    pkg-config

    # Tauri dependencies for Linux
    webkitgtk_4_1
    gtk3
    glib
    glib.dev
    cairo
    pango
    gdk-pixbuf
    libsoup_3
    openssl

    # Additional Tauri v2 deps
    librsvg
    alsa-lib

    # Audio (PipeWire integration)
    pipewire

    # Libraries needed by cargo-tauri binary
    bzip2
    zlib

    # Graphics libraries
    mesa

    # Node.js for frontend
    nodejs_22

    # Soundfonts
    soundfont-fluid

    # Windows cross-compilation
    llvmPackages.clang
    llvmPackages.lld
    llvmPackages.llvm
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
      pkgs.pipewire
    ]}:$LD_LIBRARY_PATH"

    # Configure ALSA to use PipeWire
    export ALSA_PLUGIN_DIR="${pkgs.pipewire}/lib/alsa-lib"
    export ALSA_CONFIG_PATH="${alsaConf}"

    # Set soundfont path for ordiseq synth
    export SOUNDFONT_PATH="${pkgs.soundfont-fluid}/share/soundfonts/FluidR3_GM2-2.sf2"

    # WebKitGTK workarounds for EGL issues
    export WEBKIT_DISABLE_DMABUF_RENDERER=1
    export WEBKIT_DISABLE_COMPOSITING_MODE=1
    export LIBGL_ALWAYS_SOFTWARE=1

    echo "ordiseq_app dev shell"
    echo "Run: cargo tauri dev"
    echo "Soundfont: $SOUNDFONT_PATH"
  '';
}
