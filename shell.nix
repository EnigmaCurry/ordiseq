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
    # Rust toolchain
    rustup

    # Tauri dependencies (Linux)
    pkg-config
    openssl
    glib
    gtk3
    libsoup_3
    webkitgtk_4_1

    # Audio (ALSA + PipeWire)
    alsa-lib
    pipewire

    # Node.js for frontend
    nodejs

    # Soundfonts
    soundfont-fluid
  ];

  shellHook = ''
    # Configure ALSA to use PipeWire
    export ALSA_PLUGIN_DIR="${pkgs.pipewire}/lib/alsa-lib"
    export ALSA_CONFIG_PATH="${alsaConf}"

    # Set soundfont path for ordiseq synth
    export SOUNDFONT_PATH="${pkgs.soundfont-fluid}/share/soundfonts/FluidR3_GM2-2.sf2"

    echo "ordiseq development shell"
    echo "Soundfont: $SOUNDFONT_PATH"
    echo "ALSA config: $ALSA_CONFIG_PATH"
    echo "ALSA plugins: $ALSA_PLUGIN_DIR"
  '';

  # Prevent Cargo from downloading below nix store
  CARGO_HOME = toString ./.cargo;
}
