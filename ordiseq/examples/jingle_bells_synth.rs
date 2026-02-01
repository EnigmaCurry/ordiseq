use ordiseq::prelude::*;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::env;
use std::fs::File;
use std::io::Cursor;
use std::sync::Arc;

/// Jingle Bells - synthesized to WAV using rustysynth
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    // Get the SoundFont path from command line args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <soundfont.sf2> [output.wav]", args[0]);
        eprintln!("\nExample SoundFonts:");
        eprintln!("  - TimGM6mb.sf2 (small, ~6MB)");
        eprintln!("  - FluidR3_GM.sf2 (full GM, ~140MB)");
        std::process::exit(1);
    }
    let sf2_path = &args[1];
    let output_path = args.get(2).map(|s| s.as_str()).unwrap_or("jingle_bells.wav");

    // Build the sequence (same as jingle_bells.rs)
    let time_signature = common_time(); // 4/4 96tpqn
    let mut seq = Sequence::new("Jingle Bells", time_signature)?;

    let note = NoteOrRest::Note;
    let v = 0.7; // Constant velocity for all notes
    let r = 0.5; // Note release 1.==legato, <1.==staccato
    let verse = vec![
        // Note, Beats, Velocity, Release
        (note(E), 2, v, r),
        (note(E), 2, v, r),
        (note(E), 4, v, r), // "jin-gle bells"
        (note(E), 2, v, r),
        (note(E), 2, v, r),
        (note(E), 4, v, r), // "jin-gle bells"
        (note(E), 2, v, r),
        (note(G), 2, v, r),
        (note(C), 3, v, r),
        (note(D), 1, v, r),
        (note(E), 8, v, r), // "jin-gle all the way"
        (NoteOrRest::Rest, 8, 0., 0.),
        (note(F), 2, v, r),
        (note(F), 2, v, r),
        (note(F), 3, v, r),
        (note(F), 1, v, r),
        (note(F), 2, v, r),
        (note(E), 2, v, r),
        (note(E), 2, v, r), // "Oh what fun it is to ride"
        (note(E), 1, v, r),
        (note(E), 1, v, r),
        (note(E), 2, v, r),
        (note(D), 2, v, r),
        (note(D), 2, v, r),
        (note(E), 2, v, r),
        (note(D), 4, v, r), // "in a one-horse open sleigh"
        (note(G), 4, v, r), // "hey"
    ];

    seq.load(&verse)?;
    seq = seq.transpose(7)?;

    // Convert sequence to MIDI in memory
    let smf = seq.to_midi();
    let mut midi_buffer = Vec::new();
    smf.write_std(&mut midi_buffer)?;

    // Load the SoundFont
    println!("Loading SoundFont: {}", sf2_path);
    let mut sf2_file = File::open(sf2_path)?;
    let sound_font = Arc::new(SoundFont::new(&mut sf2_file)?);

    // Load the MIDI from the buffer
    let mut midi_cursor = Cursor::new(midi_buffer);
    let midi_file = Arc::new(MidiFile::new(&mut midi_cursor)?);

    // Create the synthesizer and sequencer
    let sample_rate = 44100;
    let settings = SynthesizerSettings::new(sample_rate);
    let synthesizer = Synthesizer::new(&sound_font, &settings)?;
    let mut sequencer = MidiFileSequencer::new(synthesizer);

    // Play the MIDI file
    sequencer.play(&midi_file, false);

    // Calculate the output buffer size
    let duration_seconds = midi_file.get_length();
    let sample_count = ((sample_rate as f64) * duration_seconds) as usize;
    println!(
        "Synthesizing {:.2} seconds of audio ({} samples)",
        duration_seconds, sample_count
    );

    // Render the audio
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];
    sequencer.render(&mut left, &mut right);

    // Write to WAV file using hound
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(output_path, spec)?;

    // Interleave left and right channels
    for (l, r) in left.iter().zip(right.iter()) {
        // Convert f32 [-1.0, 1.0] to i16
        let l_sample = (*l * 32767.0).clamp(-32768.0, 32767.0) as i16;
        let r_sample = (*r * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(l_sample)?;
        writer.write_sample(r_sample)?;
    }
    writer.finalize()?;

    println!("Saved to: {}", output_path);

    Ok(())
}
