//! WAV file rendering.

use super::error::SynthError;
use super::soundfont::SoundFontSource;
use crate::sequence::Sequence;
use hound::{SampleFormat, WavSpec, WavWriter};
use rustysynth::{MidiFile, MidiFileSequencer, Synthesizer, SynthesizerSettings};
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

/// Default sample rate for WAV rendering (44.1 kHz).
pub const DEFAULT_SAMPLE_RATE: u32 = 44100;

/// Render a sequence to a WAV file.
///
/// # Arguments
///
/// * `sequence` - The sequence to render
/// * `soundfont` - The soundfont to use for synthesis
/// * `output_path` - Path to write the WAV file
/// * `sample_rate` - Sample rate in Hz (use `None` for default 44100)
///
/// # Example
///
/// ```no_run
/// use ordiseq::prelude::*;
/// use ordiseq::synth::{SoundFontSource, render_to_wav};
///
/// let soundfont = SoundFontSource::default_search().unwrap();
/// let mut seq = Sequence::new("Example", common_time()).unwrap();
/// // ... add notes ...
///
/// render_to_wav(&seq, &soundfont, "output.wav", None).unwrap();
/// ```
pub fn render_to_wav<P: AsRef<Path>>(
    sequence: &Sequence,
    soundfont: &SoundFontSource,
    output_path: P,
    sample_rate: Option<u32>,
) -> Result<(), SynthError> {
    let smf = sequence.to_midi();
    let mut midi_buffer = Vec::new();
    smf.write_std(&mut midi_buffer)?;

    render_midi_bytes_to_wav(&midi_buffer, soundfont, output_path, sample_rate)
}

/// Render raw MIDI data to a WAV file.
pub fn render_midi_bytes_to_wav<P: AsRef<Path>>(
    midi_data: &[u8],
    soundfont: &SoundFontSource,
    output_path: P,
    sample_rate: Option<u32>,
) -> Result<(), SynthError> {
    let sample_rate = sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE);

    // Load the MIDI from the buffer
    let mut midi_cursor = Cursor::new(midi_data);
    let midi_file = Arc::new(
        MidiFile::new(&mut midi_cursor).map_err(|e| SynthError::MidiParseError(e.to_string()))?,
    );
    let duration_seconds = midi_file.get_length();

    // Calculate total samples needed
    let total_samples = (sample_rate as f64 * duration_seconds).ceil() as usize;

    log::info!(
        "Rendering {:.2} seconds of audio to WAV ({}Hz, {} samples)",
        duration_seconds,
        sample_rate,
        total_samples
    );

    // Create the synthesizer and sequencer
    let settings = SynthesizerSettings::new(sample_rate as i32);
    let synthesizer = Synthesizer::new(soundfont.inner(), &settings)
        .map_err(|e| SynthError::MidiParseError(e.to_string()))?;
    let mut sequencer = MidiFileSequencer::new(synthesizer);
    sequencer.play(&midi_file, false);

    // Render audio
    let mut left = vec![0f32; total_samples];
    let mut right = vec![0f32; total_samples];
    sequencer.render(&mut left, &mut right);

    // Write to WAV file
    let spec = WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(output_path, spec)?;

    // Interleave and convert to i16
    for (l, r) in left.iter().zip(right.iter()) {
        // Clamp and convert to i16
        let l_sample = (*l * 32767.0).clamp(-32768.0, 32767.0) as i16;
        let r_sample = (*r * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(l_sample)?;
        writer.write_sample(r_sample)?;
    }

    writer.finalize()?;
    log::info!("WAV file written successfully.");

    Ok(())
}

/// Render a sequence to WAV data in memory.
///
/// Returns the WAV data as a byte vector.
pub fn render_to_wav_bytes(
    sequence: &Sequence,
    soundfont: &SoundFontSource,
    sample_rate: Option<u32>,
) -> Result<Vec<u8>, SynthError> {
    let smf = sequence.to_midi();
    let mut midi_buffer = Vec::new();
    smf.write_std(&mut midi_buffer)?;

    render_midi_bytes_to_wav_bytes(&midi_buffer, soundfont, sample_rate)
}

/// Render raw MIDI data to WAV data in memory.
pub fn render_midi_bytes_to_wav_bytes(
    midi_data: &[u8],
    soundfont: &SoundFontSource,
    sample_rate: Option<u32>,
) -> Result<Vec<u8>, SynthError> {
    let sample_rate = sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE);

    // Load the MIDI from the buffer
    let mut midi_cursor = Cursor::new(midi_data);
    let midi_file = Arc::new(
        MidiFile::new(&mut midi_cursor).map_err(|e| SynthError::MidiParseError(e.to_string()))?,
    );
    let duration_seconds = midi_file.get_length();

    // Calculate total samples needed
    let total_samples = (sample_rate as f64 * duration_seconds).ceil() as usize;

    // Create the synthesizer and sequencer
    let settings = SynthesizerSettings::new(sample_rate as i32);
    let synthesizer = Synthesizer::new(soundfont.inner(), &settings)
        .map_err(|e| SynthError::MidiParseError(e.to_string()))?;
    let mut sequencer = MidiFileSequencer::new(synthesizer);
    sequencer.play(&midi_file, false);

    // Render audio
    let mut left = vec![0f32; total_samples];
    let mut right = vec![0f32; total_samples];
    sequencer.render(&mut left, &mut right);

    // Write to in-memory WAV
    let spec = WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut buffer, spec)?;

        for (l, r) in left.iter().zip(right.iter()) {
            let l_sample = (*l * 32767.0).clamp(-32768.0, 32767.0) as i16;
            let r_sample = (*r * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(l_sample)?;
            writer.write_sample(r_sample)?;
        }

        writer.finalize()?;
    }

    Ok(buffer.into_inner())
}
