use clap::{Parser, ValueEnum};
use midly::{MetaMessage, Smf, TrackEvent, TrackEventKind};
use ordiseq::midi::HasMidiValue;
use ordiseq::prelude::*;
use ordiseq::synth::{PREFERRED_SOUNDFONTS, SoundFontSource};
use rand::Rng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use scale_omnibus::{get_scale, get_scale_names};

const ALL_PATTERNS: [Pattern; 16] = [
    Pattern::Up,
    Pattern::Down,
    Pattern::UpDown,
    Pattern::DownUp,
    Pattern::Random,
    Pattern::Converge,
    Pattern::Diverge,
    Pattern::PingPong,
    Pattern::Thirds,
    Pattern::Skip,
    Pattern::Walk,
    Pattern::Pairs,
    Pattern::Triplets,
    Pattern::Spiral,
    Pattern::Leaps,
    Pattern::Zipper,
];

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
enum Pattern {
    /// Ascending order
    Up,
    /// Descending order
    Down,
    /// Ascending then descending
    UpDown,
    /// Descending then ascending
    DownUp,
    /// Random order (reshuffled each loop)
    Random,
    /// Outside-in: alternates low/high moving inward
    Converge,
    /// Inside-out: starts from middle, expands outward
    Diverge,
    /// Bounces at ends with overlap (1,2,3,2,3,4,3,4,5...)
    PingPong,
    /// Broken thirds (1,3,2,4,3,5,4,6...)
    Thirds,
    /// Every other note, then fill gaps (1,3,5,7,2,4,6,8)
    Skip,
    /// Two steps forward, one back (1,2,1,2,3,2,3,4,3...)
    Walk,
    /// Pairs of adjacent notes alternating (1,2,1,2,3,4,3,4...)
    Pairs,
    /// Groups of 3 with overlap (1,2,3,2,3,4,3,4,5...)
    Triplets,
    /// Alternates: 1st, last, 2nd, 2nd-last, etc then reverses
    Spiral,
    /// Plays note, skips 2, plays, skips 2... then fills
    Leaps,
    /// Interleaved ascending from both ends meeting in middle
    Zipper,
}

#[derive(Parser)]
#[command(name = "scale_arp")]
#[command(about = "Play any scale as an arpeggio. Run with no args for infinite random demo mode.")]
#[command(version)]
struct Cli {
    /// Scale name (case-insensitive). If not specified, picks random scale for demo mode.
    #[arg(short, long)]
    scale: Option<String>,

    /// Root note as MIDI value (60 = C4) or note name (e.g., "C4", "A3"). Random if not specified.
    #[arg(short = 'r', long)]
    root: Option<String>,

    /// Arpeggio pattern. If not specified with scale/root, uses 4 random patterns in demo mode.
    #[arg(short = 'p', long, value_enum)]
    pattern: Option<Pattern>,

    /// Number of times to loop the pattern (ignored in demo mode)
    #[arg(short, long, default_value = "1")]
    loops: u32,

    /// Beats per note (1 = quarter note at tempo)
    #[arg(short, long, default_value = "1")]
    beats: u32,

    /// Number of octaves to span (2 = double the range)
    #[arg(short, long, default_value = "1")]
    octaves: u32,

    /// Tempo in beats per minute
    #[arg(short = 't', long, default_value = "120")]
    bpm: u32,

    /// Path to a SoundFont file (searches system directories if not provided)
    soundfont: Option<String>,
}

fn parse_root_note(s: &str) -> Result<Note, String> {
    // Try parsing as MIDI value first
    if let Ok(midi) = s.parse::<u8>() {
        return midi_to_note(midi);
    }

    // Try parsing as note name (e.g., "C4", "A#3", "Bb5")
    Note::parse(s).map_err(|e| format!("Invalid note '{}': {:?}", s, e))
}

fn midi_to_note(midi: u8) -> Result<Note, String> {
    if midi > 127 {
        return Err(format!("MIDI value {} out of range (0-127)", midi));
    }
    let octave_val = (midi / 12).saturating_sub(1);
    let pitch_val = midi % 12;

    let pitch = Pitch::try_from(pitch_val).map_err(|e| format!("Invalid pitch: {:?}", e))?;
    let named_pitch = NamedPitch::from(pitch);
    let octave = Octave::try_from(octave_val).map_err(|e| format!("Invalid octave: {:?}", e))?;

    Ok(Note::new(named_pitch, octave))
}

fn note_from_midi_offset(root: &Note, semitones: u8) -> Note {
    let root_midi = root.midi_value();
    let target_midi = root_midi.saturating_add(semitones);
    midi_to_note(target_midi).expect("Note out of range")
}

fn random_root_note() -> Note {
    let mut rng = thread_rng();
    // Pick a random root between C3 (48) and C5 (72)
    let midi: u8 = rng.gen_range(48..=72);
    midi_to_note(midi).expect("Invalid random MIDI note")
}

fn random_scale_name() -> String {
    let mut rng = thread_rng();
    let names = get_scale_names();
    names.choose(&mut rng).unwrap().clone()
}

fn random_patterns(count: usize) -> Vec<Pattern> {
    let mut rng = thread_rng();
    let mut patterns: Vec<Pattern> = ALL_PATTERNS.to_vec();
    patterns.shuffle(&mut rng);
    patterns.into_iter().take(count).collect()
}

fn note_name(note: &Note) -> String {
    format!("{:?}{}", note.named_pitch(), note.octave() as u8)
}

/// Convert BPM to MIDI tempo (microseconds per quarter note)
fn bpm_to_tempo(bpm: u32) -> u32 {
    60_000_000 / (bpm * 2)
}

/// Add tempo to MIDI bytes by inserting a tempo meta event
fn add_tempo_to_midi_bytes(midi_bytes: &[u8], bpm: u32) -> Vec<u8> {
    let tempo_us = bpm_to_tempo(bpm);

    // Parse the MIDI file into an owned version
    let smf = Smf::parse(midi_bytes).expect("Failed to parse MIDI");
    let smf_owned = smf.make_static();

    // Create new tracks with tempo inserted
    let mut new_tracks: Vec<Vec<TrackEvent<'static>>> = Vec::new();

    for track in smf_owned.tracks {
        let mut new_track: Vec<TrackEvent<'static>> = Vec::new();
        let mut tempo_added = false;

        for event in track {
            // Insert tempo after the first event (usually time signature)
            if !tempo_added && !new_track.is_empty() {
                new_track.push(TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Meta(MetaMessage::Tempo(tempo_us.into())),
                });
                tempo_added = true;
            }

            new_track.push(event);
        }

        new_tracks.push(new_track);
    }

    let new_smf: Smf<'static> = Smf {
        header: smf_owned.header,
        tracks: new_tracks,
    };

    let mut output = Vec::new();
    new_smf
        .write_std(&mut output)
        .expect("Failed to write MIDI");
    output
}

/// Build the base scale notes across octaves
fn build_base_notes(root: &Note, scale_notes: &[u8], octaves: u32) -> Vec<Note> {
    let mut notes: Vec<Note> = Vec::new();
    for octave in 0..octaves {
        let octave_offset = octave as u8 * 12;
        for &offset in scale_notes {
            let total_offset = octave_offset.saturating_add(offset);
            notes.push(note_from_midi_offset(root, total_offset));
        }
    }
    // Add the final root note at the top octave
    notes.push(note_from_midi_offset(root, octaves as u8 * 12));
    notes
}

/// Apply a pattern algorithm to reorder notes
fn apply_pattern(notes: &[Note], pattern: Pattern) -> Vec<Note> {
    let n = notes.len();
    if n == 0 {
        return vec![];
    }

    match pattern {
        Pattern::Up => notes.to_vec(),

        Pattern::Down => {
            let mut result = notes.to_vec();
            result.reverse();
            result
        }

        Pattern::UpDown => {
            let mut result = notes.to_vec();
            let mut down: Vec<Note> = notes.to_vec();
            down.reverse();
            // Skip first of descending to avoid double-play
            result.extend(down.into_iter().skip(1));
            result
        }

        Pattern::DownUp => {
            let mut result = notes.to_vec();
            result.reverse();
            // Skip first of ascending to avoid double-play
            result.extend(notes.iter().skip(1).cloned());
            result
        }

        Pattern::Random => {
            let mut result = notes.to_vec();
            result.shuffle(&mut thread_rng());
            result
        }

        Pattern::Converge => {
            // Outside-in: low, high, low+1, high-1, ...
            let mut result = Vec::with_capacity(n);
            let mut lo = 0;
            let mut hi = n - 1;
            while lo <= hi {
                result.push(notes[lo].clone());
                if lo != hi {
                    result.push(notes[hi].clone());
                }
                lo += 1;
                if hi == 0 {
                    break;
                }
                hi -= 1;
            }
            result
        }

        Pattern::Diverge => {
            // Inside-out: start from middle, alternate outward
            let mut result = Vec::with_capacity(n);
            let mid = n / 2;
            result.push(notes[mid].clone());

            for offset in 1..=mid {
                if mid + offset < n {
                    result.push(notes[mid + offset].clone());
                }
                if mid >= offset {
                    result.push(notes[mid - offset].clone());
                }
            }
            result
        }

        Pattern::PingPong => {
            // Bouncing pattern: 1,2,3,2,3,4,3,4,5...
            if n <= 2 {
                return notes.to_vec();
            }
            let mut result = Vec::new();
            let mut i = 0;
            let mut going_up = true;

            while result.len() < n * 2 - 2 {
                result.push(notes[i].clone());
                if going_up {
                    if i >= n - 1 {
                        going_up = false;
                        i -= 1;
                    } else {
                        i += 1;
                    }
                } else if i == 0 {
                    going_up = true;
                    i += 1;
                } else {
                    i -= 1;
                }
            }
            result
        }

        Pattern::Thirds => {
            // Broken thirds: 1,3,2,4,3,5,4,6...
            let mut result = Vec::new();
            let mut i = 0;
            while i < n {
                result.push(notes[i].clone());
                if i + 2 < n {
                    result.push(notes[i + 2].clone());
                }
                i += 1;
            }
            result
        }

        Pattern::Skip => {
            // Every other note, then fill: 1,3,5,7,2,4,6,8
            let mut result = Vec::new();
            // Odd indices (0, 2, 4, ...)
            for i in (0..n).step_by(2) {
                result.push(notes[i].clone());
            }
            // Even indices (1, 3, 5, ...)
            for i in (1..n).step_by(2) {
                result.push(notes[i].clone());
            }
            result
        }

        Pattern::Walk => {
            // Two forward, one back: 1,2,1,2,3,2,3,4...
            let mut result = Vec::new();
            let mut i = 0;
            while i < n {
                result.push(notes[i].clone());
                if i + 1 < n {
                    result.push(notes[i + 1].clone());
                    result.push(notes[i].clone());
                }
                i += 1;
            }
            result
        }

        Pattern::Pairs => {
            // Adjacent pairs: 1,2,1,2, 3,4,3,4, ...
            let mut result = Vec::new();
            for chunk in notes.chunks(2) {
                // Play the pair twice
                result.extend(chunk.iter().cloned());
                result.extend(chunk.iter().cloned());
            }
            result
        }

        Pattern::Triplets => {
            // Groups of 3 with overlap: 1,2,3, 2,3,4, 3,4,5...
            let mut result = Vec::new();
            for i in 0..n.saturating_sub(2) {
                result.push(notes[i].clone());
                result.push(notes[i + 1].clone());
                result.push(notes[i + 2].clone());
            }
            // Add remaining notes if less than 3
            if n == 1 {
                result.push(notes[0].clone());
            } else if n == 2 {
                result.push(notes[0].clone());
                result.push(notes[1].clone());
            }
            result
        }

        Pattern::Spiral => {
            // Converge then diverge back out
            let converged = apply_pattern(notes, Pattern::Converge);
            let mut result = converged.clone();
            let mut diverged = converged;
            diverged.reverse();
            result.extend(diverged.into_iter().skip(1));
            result
        }

        Pattern::Leaps => {
            // Skip 2, then fill: 1,4,7, 2,5,8, 3,6,9
            let mut result = Vec::new();
            for start in 0..3 {
                for i in (start..n).step_by(3) {
                    result.push(notes[i].clone());
                }
            }
            result
        }

        Pattern::Zipper => {
            // Interleave from both ends: 1,8,2,7,3,6,4,5
            let mut result = Vec::new();
            let mut lo = 0;
            let mut hi = n - 1;
            let mut take_lo = true;
            while lo <= hi {
                if take_lo {
                    result.push(notes[lo].clone());
                    lo += 1;
                } else {
                    result.push(notes[hi].clone());
                    if hi == 0 {
                        break;
                    }
                    hi -= 1;
                }
                take_lo = !take_lo;
            }
            result
        }
    }
}

fn build_scale_notes(
    root: &Note,
    scale_notes: &[u8],
    pattern: Pattern,
    loops: u32,
    beats: u32,
    octaves: u32,
) -> Vec<(NoteOrRest, u32, f32, f32)> {
    let velocity = 0.7;
    let release = 0.8;

    let base_notes = build_base_notes(root, scale_notes, octaves);

    let mut result = Vec::new();

    for _ in 0..loops {
        let pattern_notes = apply_pattern(&base_notes, pattern);

        for note in pattern_notes {
            result.push((NoteOrRest::Note(note), beats, velocity, release));
        }
    }

    result
}

fn get_scale_notes(scale_name: &str) -> Result<Vec<u8>, String> {
    let scale = get_scale(scale_name).map_err(|e| e.to_string())?;

    // Prefer regular notes, fall back to ascending for asymmetric scales
    let notes: Vec<u8> = if let Some(ref n) = scale.notes {
        n.clone()
    } else if let Some(ref n) = scale.notes_ascending {
        n.clone()
    } else {
        return Err(format!("Scale '{}' has no note data", scale_name));
    };

    Ok(notes)
}

fn load_soundfont(path: Option<&String>) -> Result<SoundFontSource, Box<dyn std::error::Error>> {
    if let Some(path) = path {
        Ok(SoundFontSource::from_path(path)?)
    } else {
        match SoundFontSource::default_search() {
            Ok(sf) => Ok(sf),
            Err(_) => {
                eprintln!("No soundfont provided and none found in system directories.");
                eprintln!("Usage: scale_arp [OPTIONS] [SOUNDFONT]");
                eprintln!("\nSearched directories:");
                for dir in SoundFontSource::default_search_dirs() {
                    eprintln!("  - {}", dir.display());
                }
                eprintln!("\nPreferred soundfonts: {:?}", PREFERRED_SOUNDFONTS);
                std::process::exit(1);
            }
        }
    }
}

/// Play a sequence with the specified BPM
fn play_sequence_with_bpm(
    player: &Player,
    sequence: &Sequence,
    bpm: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let smf = sequence.to_midi();

    let mut midi_buffer = Vec::new();
    smf.write_std(&mut midi_buffer)?;

    let midi_with_tempo = add_tempo_to_midi_bytes(&midi_buffer, bpm);

    player.play_midi_bytes(&midi_with_tempo)?;
    Ok(())
}

fn run_demo_mode(
    soundfont: SoundFontSource,
    beats: u32,
    octaves: u32,
    bpm: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Using SoundFont: {}", soundfont.path().display());
    println!("\n=== DEMO MODE ===");
    println!("Playing random scales with random patterns in infinite loop.");
    println!("BPM: {} | Press Ctrl+C to stop.\n", bpm);

    let player = Player::new(soundfont)?;

    loop {
        // Pick random scale and root
        let scale_name = random_scale_name();
        let root = random_root_note();

        // Try to load the scale, skip if it fails
        let scale_notes = match get_scale_notes(&scale_name) {
            Ok(notes) => notes,
            Err(_) => continue, // Skip scales without note data
        };

        let scale = get_scale(&scale_name)?;

        // Pick 4 random patterns
        let patterns = random_patterns(4);

        println!(
            "Scale: {} | Root: {} | Octaves: {}",
            scale.name,
            note_name(&root),
            octaves
        );
        println!(
            "Patterns: {:?} (each x2)",
            patterns
                .iter()
                .map(|p| format!("{:?}", p))
                .collect::<Vec<_>>()
        );

        // Build all patterns into one continuous sequence
        let time_signature = common_time();
        let mut seq = Sequence::new(&format!("{} Arpeggio", scale.name), time_signature)?;

        let mut all_notes: Vec<(NoteOrRest, u32, f32, f32)> = Vec::new();
        for pattern in &patterns {
            // Each pattern plays twice
            let pattern_notes = build_scale_notes(&root, &scale_notes, *pattern, 2, beats, octaves);
            all_notes.extend(pattern_notes);
        }

        seq.load(&all_notes)?;

        print!("  Playing... ");
        std::io::Write::flush(&mut std::io::stdout())?;

        play_sequence_with_bpm(&player, &seq, bpm)?;
        println!("done\n");
        std::thread::sleep(Duration::from_secs(2));
    }
}

fn run_single_mode(
    soundfont: SoundFontSource,
    scale_name: &str,
    root: Note,
    pattern: Pattern,
    loops: u32,
    beats: u32,
    octaves: u32,
    bpm: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Using SoundFont: {}", soundfont.path().display());
    println!(
        "Root note: {} (MIDI {})",
        note_name(&root),
        root.midi_value()
    );

    let scale_notes = get_scale_notes(scale_name)?;
    let scale = get_scale(scale_name)?;
    println!("Scale: {} - notes: {:?}", scale.name, scale_notes);
    println!(
        "Pattern: {:?}, Loops: {}, Beats per note: {}, Octaves: {}, BPM: {}",
        pattern, loops, beats, octaves, bpm
    );

    let time_signature = common_time();
    let mut seq = Sequence::new(&format!("{} Scale Arpeggio", scale.name), time_signature)?;

    let notes = build_scale_notes(&root, &scale_notes, pattern, loops, beats, octaves);
    seq.load(&notes)?;

    let player = Player::new(soundfont)?;
    play_sequence_with_bpm(&player, &seq, bpm)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let cli = Cli::parse();

    let soundfont = load_soundfont(cli.soundfont.as_ref())?;

    // Determine if we're in demo mode (no scale, root, or pattern specified)
    let demo_mode = cli.scale.is_none() && cli.root.is_none() && cli.pattern.is_none();

    if demo_mode {
        run_demo_mode(soundfont, cli.beats, cli.octaves, cli.bpm)?;
    } else {
        // Use defaults for any unspecified options
        let scale_name = cli.scale.unwrap_or_else(|| "major".to_string());
        let root = if let Some(ref r) = cli.root {
            parse_root_note(r)?
        } else {
            parse_root_note("C4")?
        };
        let pattern = cli.pattern.unwrap_or(Pattern::Up);

        run_single_mode(
            soundfont,
            &scale_name,
            root,
            pattern,
            cli.loops,
            cli.beats,
            cli.octaves,
            cli.bpm,
        )?;
    }

    Ok(())
}
