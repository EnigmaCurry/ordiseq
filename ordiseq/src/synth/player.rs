//! Audio playback using cpal.

use super::error::SynthError;
use super::soundfont::SoundFontSource;
use crate::sequence::Sequence;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustysynth::{MidiFile, MidiFileSequencer, Synthesizer, SynthesizerSettings};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

/// Audio player for MIDI sequences.
///
/// The player holds a soundfont and can play sequences through the default audio device.
pub struct Player {
    soundfont: SoundFontSource,
}

impl Player {
    /// Create a new player with the given soundfont.
    pub fn new(soundfont: SoundFontSource) -> Result<Self, SynthError> {
        Ok(Self { soundfont })
    }

    /// Create a new player, searching for a default soundfont.
    pub fn with_default_soundfont() -> Result<Self, SynthError> {
        let soundfont = SoundFontSource::default_search()?;
        Self::new(soundfont)
    }

    /// Get the soundfont being used by this player.
    pub fn soundfont(&self) -> &SoundFontSource {
        &self.soundfont
    }

    /// Play a sequence through the default audio device.
    ///
    /// This blocks until playback is complete.
    pub fn play_sequence(&self, sequence: &Sequence) -> Result<(), SynthError> {
        let smf = sequence.to_midi();
        let mut midi_buffer = Vec::new();
        smf.write_std(&mut midi_buffer)?;
        self.play_midi_bytes(&midi_buffer)
    }

    /// Play raw MIDI data through the default audio device.
    ///
    /// This blocks until playback is complete.
    pub fn play_midi_bytes(&self, midi_data: &[u8]) -> Result<(), SynthError> {
        // Load the MIDI from the buffer
        let mut midi_cursor = Cursor::new(midi_data);
        let midi_file = Arc::new(
            MidiFile::new(&mut midi_cursor)
                .map_err(|e| SynthError::MidiParseError(e.to_string()))?,
        );
        let duration_seconds = midi_file.get_length();

        // Set up audio output
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(SynthError::NoAudioDevice)?;

        let device_name = device
            .name()
            .map_err(|e| SynthError::DeviceNameError(e.to_string()))?;
        log::info!("Using audio device: {}", device_name);

        let config = device
            .default_output_config()
            .map_err(|e| SynthError::DefaultConfigError(e.to_string()))?;

        let sample_rate = config.sample_rate().0 as i32;
        let channels = config.channels() as usize;

        log::info!(
            "Playing {:.2} seconds of audio ({}Hz, {} channels)",
            duration_seconds,
            sample_rate,
            channels
        );

        // Create the synthesizer and sequencer
        let settings = SynthesizerSettings::new(sample_rate);
        let synthesizer = Synthesizer::new(self.soundfont.inner(), &settings)
            .map_err(|e| SynthError::MidiParseError(e.to_string()))?;
        let mut sequencer = MidiFileSequencer::new(synthesizer);
        sequencer.play(&midi_file, false);

        // Wrap sequencer in Arc<Mutex> for sharing with audio callback
        let sequencer = Arc::new(Mutex::new(sequencer));
        let sequencer_clone = Arc::clone(&sequencer);

        // Track playback completion
        let samples_played = Arc::new(Mutex::new(0usize));
        let samples_played_clone = Arc::clone(&samples_played);
        let total_samples = (sample_rate as f64 * duration_seconds) as usize;

        // Build the audio stream
        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut seq = sequencer_clone.lock().unwrap();
                    let frames = data.len() / channels;

                    // Render audio from the synthesizer
                    let mut left = vec![0f32; frames];
                    let mut right = vec![0f32; frames];
                    seq.render(&mut left, &mut right);

                    // Interleave into output buffer
                    for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
                        let base = i * channels;
                        if channels >= 2 {
                            data[base] = *l;
                            data[base + 1] = *r;
                        } else {
                            // Mono: mix left and right
                            data[base] = (*l + *r) / 2.0;
                        }
                    }

                    // Track progress
                    let mut played = samples_played_clone.lock().unwrap();
                    *played += frames;
                },
                |err| log::error!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| SynthError::StreamBuildError(e.to_string()))?;

        stream
            .play()
            .map_err(|e| SynthError::StreamPlayError(e.to_string()))?;

        // Wait for playback to complete
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let played = *samples_played.lock().unwrap();
            if played >= total_samples {
                // Add a small buffer to let the last samples play out
                std::thread::sleep(std::time::Duration::from_millis(200));
                break;
            }
        }

        log::info!("Playback complete.");
        Ok(())
    }
}

impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player")
            .field("soundfont", &self.soundfont)
            .finish()
    }
}
