//! Synthesizer module for audio playback and rendering.
//!
//! This module provides soundfont loading, MIDI playback, and WAV export capabilities.
//!
//! # Features
//!
//! This module is only available when the `synth` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! ordiseq = { version = "0.1", features = ["synth"] }
//! ```
//!
//! # Example
//!
//! ```no_run
//! use ordiseq::prelude::*;
//! use ordiseq::synth::{SoundFontSource, Player};
//!
//! // Find and load a soundfont
//! let soundfont = SoundFontSource::default_search()
//!     .expect("No soundfont found");
//!
//! // Create a sequence
//! let mut seq = Sequence::new("Example", common_time()).unwrap();
//! // ... add notes ...
//!
//! // Play the sequence
//! let player = Player::new(soundfont).unwrap();
//! player.play_sequence(&seq).unwrap();
//! ```

mod error;
mod player;
mod soundfont;
mod wav;

pub use error::SynthError;
pub use player::Player;
pub use soundfont::{find_default_soundfont, SoundFontSource, PREFERRED_SOUNDFONTS, SEARCH_DIRS};
pub use wav::{
    render_midi_bytes_to_wav, render_midi_bytes_to_wav_bytes, render_to_wav, render_to_wav_bytes,
    DEFAULT_SAMPLE_RATE,
};
