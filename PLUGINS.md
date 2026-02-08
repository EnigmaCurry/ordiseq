# Ordiseq Plugin Suite

A collection of MIDI plugins built with [NIH-plug](https://github.com/robbert-vdh/nih-plug) framework.

## Plugins

### 1. Melody Player (melody_player_plug)

**Type:** MIDI Generator / Instrument
**Formats:** VST3, CLAP

A fun MIDI generator that plays classic melodies when your DAW transport is running.

#### Features
- **Switchable Melodies:** Toggle between "Jingle Bells" and "Row Row Your Boat"
- **120 BPM Playback:** Consistent tempo for both melodies
- **Transport Sync:** Only plays when DAW transport is active
- **Smooth Transitions:** Automatically resets when switching melodies

#### Usage
1. Load the plugin on a MIDI track
2. Route the MIDI output to a virtual instrument
3. Start your DAW transport
4. Use the "Melody" parameter to switch between songs

---

### 2. MIDI Inverter (midi_inverter_plug)

**Type:** MIDI Effect
**Formats:** VST3, CLAP

A creative MIDI effect that inverts incoming notes around a center point, creating mirror melodies.

#### Features
- **Adjustable Center Note:** Default is Middle C (MIDI 60), adjustable from 0-127
- **Real-time Processing:** Inverts note-on, note-off, and polyphonic aftertouch
- **Pass-through:** Other MIDI events (CC, pitch bend, etc.) pass through unchanged

#### How It Works
The plugin mirrors notes around the center point:
- If center = C4 (60):
  - D4 (62) → Bb3 (58)  _(distance +2 becomes -2)_
  - E4 (64) → Ab3 (56)  _(distance +4 becomes -4)_
  - G4 (67) → F3 (53)   _(distance +7 becomes -7)_

#### Usage
1. Load the plugin as a MIDI effect (before your instrument)
2. Play some notes or route MIDI to it
3. Adjust the "Center Note" parameter to change the inversion point
4. Experiment with different center points for different musical effects

---

## Installation

### Quick Install
Copy the bundled plugins to your DAW's plugin folder:

**Windows:**
- VST3: `C:\Program Files\Common Files\VST3\`
- CLAP: `C:\Program Files\Common Files\CLAP\`

**macOS:**
- VST3: `~/Library/Audio/Plug-Ins/VST3/`
- CLAP: `~/Library/Audio/Plug-Ins/CLAP/`

**Linux:**
- VST3: `~/.vst3/`
- CLAP: `~/.clap/`

### From Source
Build both plugins:
```bash
just build-plugins
```

Or build individually:
```bash
# Melody Player
just build-melody-player

# MIDI Inverter
just build-midi-inverter
```

Bundled plugins will be in `target/bundled/`:
- `melody_player_plug.clap` / `melody_player_plug.vst3`
- `midi_inverter_plug.clap` / `midi_inverter_plug.vst3`

## Building

### Requirements
- Rust (2024 edition or later)
- Cargo

### Build Commands

```bash
# Build both plugins (release mode)
cargo build --release --package melody_player_plug --package midi_inverter_plug

# Bundle for distribution
cargo run --package xtask --release -- bundle melody_player_plug --release
cargo run --package xtask --release -- bundle midi_inverter_plug --release

# Or use the just commands
just build-plugins
```

## Development

### Plugin Architecture
Both plugins use the NIH-plug framework which provides:
- Cross-platform VST3 and CLAP support
- Efficient MIDI processing
- Parameter automation
- State persistence

### Key Implementation Notes

**Melody Player:**
- Efficient buffer-based processing (not per-sample)
- Tracks note timing across buffer boundaries
- Detects melody changes and resets playback state
- Safety checks to prevent crashes when switching melodies

**MIDI Inverter:**
- Simple mathematical note inversion
- Processes note events in real-time
- Clamps output to valid MIDI range (0-127)
- Zero latency

## Testing

Load the plugins in your DAW:

**Melody Player Test:**
1. Create a new MIDI track
2. Load Melody Player as an instrument
3. Add a synth after it to hear the output
4. Start transport and hear Jingle Bells
5. Switch the melody parameter to hear Row Row Your Boat

**MIDI Inverter Test:**
1. Create a MIDI track with a virtual instrument
2. Insert MIDI Inverter as a MIDI effect (before the instrument)
3. Play some notes on your MIDI keyboard
4. Hear the inverted melody
5. Adjust the center note and play again

## License

MIT License - See LICENSE file for details

## Credits

Built with:
- [NIH-plug](https://github.com/robbert-vdh/nih-plug) - Plugin framework
- [Rust](https://www.rust-lang.org/) - Programming language

Part of the [Ordiseq](https://github.com/EnigmaCurry/ordiseq) project.
