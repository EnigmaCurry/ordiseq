//! Synth-specific error types.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during synthesizer operations.
#[derive(Debug, Error)]
pub enum SynthError {
    /// No soundfont file was found in any search location.
    #[error("No soundfont found. Searched directories: {searched:?}")]
    NoSoundFontFound { searched: Vec<PathBuf> },

    /// Failed to open a soundfont file.
    #[error("Failed to open soundfont at {path}: {source}")]
    SoundFontOpenError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse a soundfont file.
    #[error("Failed to parse soundfont at {path}: {message}")]
    SoundFontParseError { path: PathBuf, message: String },

    /// No audio output device was found.
    #[error("No audio output device found")]
    NoAudioDevice,

    /// Failed to get audio device name.
    #[error("Failed to get audio device name: {0}")]
    DeviceNameError(String),

    /// Failed to get default audio output configuration.
    #[error("Failed to get default output config: {0}")]
    DefaultConfigError(String),

    /// Failed to build audio stream.
    #[error("Failed to build audio stream: {0}")]
    StreamBuildError(String),

    /// Failed to play audio stream.
    #[error("Failed to play audio stream: {0}")]
    StreamPlayError(String),

    /// Failed to parse MIDI data.
    #[error("Failed to parse MIDI data: {0}")]
    MidiParseError(String),

    /// Failed to write WAV file.
    #[error("Failed to write WAV file: {0}")]
    WavWriteError(#[from] hound::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
