use clap::{Parser, ValueEnum};
use midly::{MetaMessage, MidiMessage, Smf, TrackEvent, TrackEventKind};
use ordiseq::midi::HasMidiValue;
use ordiseq::prelude::*;
use ordiseq::synth::{SoundFontSource, PREFERRED_SOUNDFONTS};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use scale_omnibus::{get_scale, get_scale_names};
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;

/// Minimum note duration in beats (prevents notes too short for soundfont to trigger)
const MIN_NOTE_BEATS: f32 = 0.2;

/// Greatest common divisor
fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Least common multiple
fn lcm(a: usize, b: usize) -> usize {
    if a == 0 || b == 0 { 0 } else { a * b / gcd(a, b) }
}

/// Calculate number of groove loops needed for pattern to align (perfect loop)
fn loops_for_alignment(groove_steps: usize, pattern_notes: usize) -> usize {
    lcm(groove_steps, pattern_notes) / groove_steps
}

// Arpeggio patterns (same as scale_arp)
const ALL_PATTERNS: [ArpPattern; 16] = [
    ArpPattern::Up,
    ArpPattern::Down,
    ArpPattern::UpDown,
    ArpPattern::DownUp,
    ArpPattern::Random,
    ArpPattern::Converge,
    ArpPattern::Diverge,
    ArpPattern::PingPong,
    ArpPattern::Thirds,
    ArpPattern::Skip,
    ArpPattern::Walk,
    ArpPattern::Pairs,
    ArpPattern::Triplets,
    ArpPattern::Spiral,
    ArpPattern::Leaps,
    ArpPattern::Zipper,
];

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
enum ArpPattern {
    Up,
    Down,
    UpDown,
    DownUp,
    Random,
    Converge,
    Diverge,
    PingPong,
    Thirds,
    Skip,
    Walk,
    Pairs,
    Triplets,
    Spiral,
    Leaps,
    Zipper,
}

/// A rhythmic groove defines note durations, velocities, and rests
#[derive(Debug, Clone)]
struct Groove {
    name: &'static str,
    /// Each step is (beats, velocity, is_rest)
    /// beats can be fractional via numerator/denominator
    steps: Vec<GrooveStep>,
}

#[derive(Debug, Clone, Copy)]
struct GrooveStep {
    /// Duration in beats (1 = quarter note)
    beats: f32,
    /// Velocity 0.0-1.0
    velocity: f32,
    /// If true, this step is a rest (skip the note)
    is_rest: bool,
}

impl GrooveStep {
    fn note(beats: f32, velocity: f32) -> Self {
        Self {
            beats,
            velocity,
            is_rest: false,
        }
    }

    fn rest(beats: f32) -> Self {
        Self {
            beats,
            velocity: 0.0,
            is_rest: true,
        }
    }
}

fn all_grooves() -> Vec<Groove> {
    vec![
        // Straight patterns
        Groove {
            name: "straight-quarters",
            steps: vec![
                GrooveStep::note(1.0, 0.8),
                GrooveStep::note(1.0, 0.6),
                GrooveStep::note(1.0, 0.7),
                GrooveStep::note(1.0, 0.6),
            ],
        },
        Groove {
            name: "straight-eighths",
            steps: vec![
                GrooveStep::note(0.5, 0.8),
                GrooveStep::note(0.5, 0.5),
                GrooveStep::note(0.5, 0.7),
                GrooveStep::note(0.5, 0.5),
                GrooveStep::note(0.5, 0.75),
                GrooveStep::note(0.5, 0.5),
                GrooveStep::note(0.5, 0.7),
                GrooveStep::note(0.5, 0.5),
            ],
        },
        // Swing feel (long-short)
        Groove {
            name: "swing",
            steps: vec![
                GrooveStep::note(0.67, 0.8),
                GrooveStep::note(0.33, 0.5),
                GrooveStep::note(0.67, 0.75),
                GrooveStep::note(0.33, 0.5),
            ],
        },
        Groove {
            name: "hard-swing",
            steps: vec![
                GrooveStep::note(0.75, 0.85),
                GrooveStep::note(0.25, 0.45),
                GrooveStep::note(0.75, 0.8),
                GrooveStep::note(0.25, 0.45),
            ],
        },
        // Syncopation patterns
        Groove {
            name: "syncopated",
            steps: vec![
                GrooveStep::note(0.5, 0.8),
                GrooveStep::rest(0.5),
                GrooveStep::note(0.5, 0.7),
                GrooveStep::note(0.5, 0.6),
                GrooveStep::rest(0.5),
                GrooveStep::note(0.5, 0.75),
            ],
        },
        Groove {
            name: "offbeat",
            steps: vec![
                GrooveStep::rest(0.5),
                GrooveStep::note(0.5, 0.75),
                GrooveStep::rest(0.5),
                GrooveStep::note(0.5, 0.7),
                GrooveStep::rest(0.5),
                GrooveStep::note(0.5, 0.75),
                GrooveStep::rest(0.5),
                GrooveStep::note(0.5, 0.7),
            ],
        },
        // Dotted rhythms
        Groove {
            name: "dotted-quarters",
            steps: vec![
                GrooveStep::note(1.5, 0.8),
                GrooveStep::note(0.5, 0.6),
                GrooveStep::note(1.5, 0.75),
                GrooveStep::note(0.5, 0.6),
            ],
        },
        Groove {
            name: "dotted-eighths",
            steps: vec![
                GrooveStep::note(0.75, 0.8),
                GrooveStep::note(0.25, 0.55),
                GrooveStep::note(0.75, 0.75),
                GrooveStep::note(0.25, 0.55),
            ],
        },
        // Triplet feels
        Groove {
            name: "triplets",
            steps: vec![
                GrooveStep::note(0.333, 0.8),
                GrooveStep::note(0.333, 0.6),
                GrooveStep::note(0.334, 0.65),
            ],
        },
        Groove {
            name: "shuffle",
            steps: vec![
                GrooveStep::note(0.667, 0.8),
                GrooveStep::note(0.333, 0.55),
            ],
        },
        // Accent patterns
        Groove {
            name: "accented-twos",
            steps: vec![
                GrooveStep::note(0.5, 0.9),
                GrooveStep::note(0.5, 0.4),
            ],
        },
        Groove {
            name: "accented-threes",
            steps: vec![
                GrooveStep::note(0.5, 0.9),
                GrooveStep::note(0.5, 0.45),
                GrooveStep::note(0.5, 0.45),
            ],
        },
        Groove {
            name: "accented-fours",
            steps: vec![
                GrooveStep::note(0.5, 0.9),
                GrooveStep::note(0.5, 0.4),
                GrooveStep::note(0.5, 0.55),
                GrooveStep::note(0.5, 0.4),
            ],
        },
        // Rest patterns (sparser)
        Groove {
            name: "sparse",
            steps: vec![
                GrooveStep::note(1.0, 0.8),
                GrooveStep::rest(1.0),
                GrooveStep::note(0.5, 0.7),
                GrooveStep::rest(0.5),
            ],
        },
        Groove {
            name: "breathing",
            steps: vec![
                GrooveStep::note(0.5, 0.8),
                GrooveStep::note(0.5, 0.7),
                GrooveStep::note(0.5, 0.65),
                GrooveStep::rest(0.5),
            ],
        },
        // Mixed/complex patterns
        Groove {
            name: "clave",
            steps: vec![
                GrooveStep::note(0.75, 0.85),
                GrooveStep::rest(0.25),
                GrooveStep::note(0.5, 0.7),
                GrooveStep::rest(0.5),
                GrooveStep::note(0.5, 0.75),
                GrooveStep::note(0.75, 0.7),
                GrooveStep::rest(0.25),
                GrooveStep::note(0.5, 0.7),
            ],
        },
        Groove {
            name: "bossa",
            steps: vec![
                GrooveStep::note(0.75, 0.75),
                GrooveStep::note(0.75, 0.6),
                GrooveStep::note(0.5, 0.7),
                GrooveStep::note(0.75, 0.6),
                GrooveStep::note(0.75, 0.7),
                GrooveStep::note(0.5, 0.6),
            ],
        },
        Groove {
            name: "funk",
            steps: vec![
                GrooveStep::note(0.25, 0.85),
                GrooveStep::rest(0.25),
                GrooveStep::note(0.25, 0.7),
                GrooveStep::note(0.25, 0.6),
                GrooveStep::rest(0.5),
                GrooveStep::note(0.25, 0.8),
                GrooveStep::note(0.25, 0.5),
            ],
        },
        Groove {
            name: "reggae",
            steps: vec![
                GrooveStep::rest(0.5),
                GrooveStep::note(0.5, 0.75),
                GrooveStep::rest(0.5),
                GrooveStep::note(0.5, 0.7),
            ],
        },
        Groove {
            name: "driving",
            steps: vec![
                GrooveStep::note(0.5, 0.85),
                GrooveStep::note(0.5, 0.7),
                GrooveStep::note(0.5, 0.75),
                GrooveStep::note(0.5, 0.7),
            ],
        },
    ]
}

#[derive(Parser)]
#[command(name = "groove_improv")]
#[command(about = "Improvisational arpeggio player with rhythmic grooves.\n\n\
Demo mode controls:\n  \
Space     - Next variation of current scale\n  \
Enter     - Next scale\n  \
Backspace - Previous\n  \
Esc       - Quit")]
#[command(version)]
struct Cli {
    /// Scale name (case-insensitive). If not specified, picks random scale for demo mode.
    #[arg(short, long)]
    scale: Option<String>,

    /// Root note as MIDI value (60 = C4) or note name (e.g., "C4", "A3"). Random if not specified.
    #[arg(short = 'r', long)]
    root: Option<String>,

    /// Arpeggio pattern. If not specified, uses random patterns in demo mode.
    #[arg(short = 'p', long, value_enum)]
    pattern: Option<ArpPattern>,

    /// Groove name. If not specified, uses random grooves in demo mode.
    #[arg(short = 'g', long)]
    groove: Option<String>,

    /// Number of times to loop the groove pattern
    #[arg(short, long, default_value = "12")]
    loops: u32,

    /// Number of octaves to span
    #[arg(short, long, default_value = "2")]
    octaves: u32,

    /// Tempo in beats per minute
    #[arg(short = 't', long, default_value = "90")]
    bpm: u32,

    /// Seed for reproducible random generation (printed during demo mode)
    #[arg(long)]
    seed: Option<u64>,

    /// Variation index (used with --seed to reproduce exact state)
    #[arg(long, default_value = "0")]
    variation: u32,

    /// GM instrument for melody (0-127). Use 'il' in demo mode to change.
    #[arg(long)]
    lead: Option<u8>,

    /// GM instrument for bass/chords (0-127). Use 'ib' in demo mode to change.
    #[arg(long)]
    bass: Option<u8>,

    /// Strum delay for bass chords in ticks (0 = no strum). Notes are offset by this amount with humanized variance.
    #[arg(long, default_value = "12")]
    strum: u32,

    /// Fill density for lead (0.0 = sparse, 1.0 = dense double-time fills). Affects note density.
    #[arg(long, default_value = "0.5")]
    fill: f32,

    /// Start with lead muted
    #[arg(long)]
    ml: bool,

    /// Start with bass muted
    #[arg(long)]
    mb: bool,

    /// Export MIDI to file instead of playing (requires --seed)
    #[arg(long)]
    output: Option<String>,

    /// List available grooves and exit
    #[arg(long)]
    list_grooves: bool,

    /// Path to a SoundFont file (searches system directories if not provided)
    soundfont: Option<String>,
}

fn parse_root_note(s: &str) -> Result<Note, String> {
    if let Ok(midi) = s.parse::<u8>() {
        return midi_to_note(midi);
    }
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

fn random_root_note<R: Rng>(rng: &mut R) -> Note {
    let midi: u8 = rng.gen_range(48..=72);
    midi_to_note(midi).expect("Invalid random MIDI note")
}

fn random_scale_name<R: Rng>(rng: &mut R) -> String {
    let mut names = get_scale_names();
    names.sort(); // Ensure deterministic order for seeded selection
    names.choose(rng).unwrap().clone()
}

fn random_pattern<R: Rng>(rng: &mut R) -> ArpPattern {
    *ALL_PATTERNS.choose(rng).unwrap()
}

fn random_groove<R: Rng>(rng: &mut R, grooves: &[Groove]) -> Groove {
    grooves.choose(rng).unwrap().clone()
}

fn note_name(note: &Note) -> String {
    format!("{:?}{}", note.named_pitch(), note.octave() as u8)
}

/// Build a triad from scale degrees (root, 3rd, 5th)
fn build_scale_triad(root: &Note, scale_notes: &[u8]) -> Vec<Note> {
    let mut chord = vec![root.clone()];
    // Get 3rd (index 2 in scale) and 5th (index 4 in scale)
    if scale_notes.len() > 2 {
        if let Ok(third) = midi_to_note(root.midi_value().saturating_add(scale_notes[2])) {
            chord.push(third);
        }
    }
    if scale_notes.len() > 4 {
        if let Ok(fifth) = midi_to_note(root.midi_value().saturating_add(scale_notes[4])) {
            chord.push(fifth);
        }
    }
    chord
}

/// Build an augmented chord (root, major 3rd +4, augmented 5th +8)
fn build_augmented_chord(root: &Note) -> Vec<Note> {
    let mut chord = vec![root.clone()];
    if let Ok(third) = midi_to_note(root.midi_value().saturating_add(4)) {
        chord.push(third);
    }
    if let Ok(fifth) = midi_to_note(root.midi_value().saturating_add(8)) {
        chord.push(fifth);
    }
    chord
}

fn bpm_to_tempo(bpm: u32) -> u32 {
    60_000_000 / bpm
}

/// General MIDI instrument names (0-127)
const GM_INSTRUMENTS: [&str; 128] = [
    // Piano (0-7)
    "Acoustic Grand Piano", "Bright Acoustic Piano", "Electric Grand Piano", "Honky-tonk Piano",
    "Electric Piano 1", "Electric Piano 2", "Harpsichord", "Clavinet",
    // Chromatic Percussion (8-15)
    "Celesta", "Glockenspiel", "Music Box", "Vibraphone",
    "Marimba", "Xylophone", "Tubular Bells", "Dulcimer",
    // Organ (16-23)
    "Drawbar Organ", "Percussive Organ", "Rock Organ", "Church Organ",
    "Reed Organ", "Accordion", "Harmonica", "Tango Accordion",
    // Guitar (24-31)
    "Acoustic Guitar (nylon)", "Acoustic Guitar (steel)", "Electric Guitar (jazz)", "Electric Guitar (clean)",
    "Electric Guitar (muted)", "Overdriven Guitar", "Distortion Guitar", "Guitar Harmonics",
    // Bass (32-39)
    "Acoustic Bass", "Electric Bass (finger)", "Electric Bass (pick)", "Fretless Bass",
    "Slap Bass 1", "Slap Bass 2", "Synth Bass 1", "Synth Bass 2",
    // Strings (40-47)
    "Violin", "Viola", "Cello", "Contrabass",
    "Tremolo Strings", "Pizzicato Strings", "Orchestral Harp", "Timpani",
    // Ensemble (48-55)
    "String Ensemble 1", "String Ensemble 2", "Synth Strings 1", "Synth Strings 2",
    "Choir Aahs", "Voice Oohs", "Synth Voice", "Orchestra Hit",
    // Brass (56-63)
    "Trumpet", "Trombone", "Tuba", "Muted Trumpet",
    "French Horn", "Brass Section", "Synth Brass 1", "Synth Brass 2",
    // Reed (64-71)
    "Soprano Sax", "Alto Sax", "Tenor Sax", "Baritone Sax",
    "Oboe", "English Horn", "Bassoon", "Clarinet",
    // Pipe (72-79)
    "Piccolo", "Flute", "Recorder", "Pan Flute",
    "Blown Bottle", "Shakuhachi", "Whistle", "Ocarina",
    // Synth Lead (80-87)
    "Lead 1 (square)", "Lead 2 (sawtooth)", "Lead 3 (calliope)", "Lead 4 (chiff)",
    "Lead 5 (charang)", "Lead 6 (voice)", "Lead 7 (fifths)", "Lead 8 (bass + lead)",
    // Synth Pad (88-95)
    "Pad 1 (new age)", "Pad 2 (warm)", "Pad 3 (polysynth)", "Pad 4 (choir)",
    "Pad 5 (bowed)", "Pad 6 (metallic)", "Pad 7 (halo)", "Pad 8 (sweep)",
    // Synth Effects (96-103)
    "FX 1 (rain)", "FX 2 (soundtrack)", "FX 3 (crystal)", "FX 4 (atmosphere)",
    "FX 5 (brightness)", "FX 6 (goblins)", "FX 7 (echoes)", "FX 8 (sci-fi)",
    // Ethnic (104-111)
    "Sitar", "Banjo", "Shamisen", "Koto",
    "Kalimba", "Bagpipe", "Fiddle", "Shanai",
    // Percussive (112-119)
    "Tinkle Bell", "Agogo", "Steel Drums", "Woodblock",
    "Taiko Drum", "Melodic Tom", "Synth Drum", "Reverse Cymbal",
    // Sound Effects (120-127)
    "Guitar Fret Noise", "Breath Noise", "Seashore", "Bird Tweet",
    "Telephone Ring", "Helicopter", "Applause", "Gunshot",
];

fn gm_instrument_name(program: u8) -> &'static str {
    GM_INSTRUMENTS[program as usize]
}

fn add_tempo_to_midi_bytes(midi_bytes: &[u8], bpm: u32) -> Vec<u8> {
    prepare_midi_bytes(midi_bytes, bpm, None, None, false, false)
}

fn prepare_midi_bytes(
    midi_bytes: &[u8],
    bpm: u32,
    lead: Option<u8>,
    bass: Option<u8>,
    mute_lead: bool,
    mute_bass: bool,
) -> Vec<u8> {
    let tempo_us = bpm_to_tempo(bpm);

    let smf = Smf::parse(midi_bytes).expect("Failed to parse MIDI");
    let smf_owned = smf.make_static();

    let mut new_tracks: Vec<Vec<TrackEvent<'static>>> = Vec::new();

    for track in smf_owned.tracks {
        let mut new_track: Vec<TrackEvent<'static>> = Vec::new();
        let mut header_added = false;
        // Track accumulated delta for muted events
        let mut accumulated_delta: u32 = 0;

        for event in track {
            if !header_added && !new_track.is_empty() {
                // Add tempo
                new_track.push(TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Meta(MetaMessage::Tempo(tempo_us.into())),
                });
                // Add program change for channel 0 (lead/melody)
                if let Some(prog) = lead {
                    new_track.push(TrackEvent {
                        delta: 0.into(),
                        kind: TrackEventKind::Midi {
                            channel: 0.into(),
                            message: MidiMessage::ProgramChange { program: prog.into() },
                        },
                    });
                }
                // Add program change for channel 1 (bass/chords)
                // Default to 33 = Electric Bass (finger) if not specified
                let bass_prog = bass.unwrap_or(33);
                new_track.push(TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Midi {
                        channel: 1.into(),
                        message: MidiMessage::ProgramChange { program: bass_prog.into() },
                    },
                });
                header_added = true;
            }

            // Check if this event should be muted (filter by channel)
            let should_mute = match &event.kind {
                TrackEventKind::Midi { channel, .. } => {
                    let ch: u8 = (*channel).into();
                    (ch == 0 && mute_lead) || (ch == 1 && mute_bass)
                }
                _ => false,
            };

            if should_mute {
                // Accumulate the delta time for the next non-muted event
                accumulated_delta += u32::from(event.delta);
            } else {
                // Add accumulated delta to this event's delta
                let adjusted_delta = u32::from(event.delta) + accumulated_delta;
                accumulated_delta = 0;
                new_track.push(TrackEvent {
                    delta: adjusted_delta.into(),
                    kind: event.kind,
                });
            }
        }

        new_tracks.push(new_track);
    }

    let new_smf: Smf<'static> = Smf {
        header: smf_owned.header,
        tracks: new_tracks,
    };

    let mut output = Vec::new();
    new_smf.write_std(&mut output).expect("Failed to write MIDI");
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
    notes.push(note_from_midi_offset(root, octaves as u8 * 12));
    notes
}

/// Apply an arpeggio pattern to reorder notes
fn apply_pattern<R: Rng>(rng: &mut R, notes: &[Note], pattern: ArpPattern) -> Vec<Note> {
    let n = notes.len();
    if n == 0 {
        return vec![];
    }

    match pattern {
        ArpPattern::Up => notes.to_vec(),
        ArpPattern::Down => {
            let mut result = notes.to_vec();
            result.reverse();
            result
        }
        ArpPattern::UpDown => {
            let mut result = notes.to_vec();
            let mut down: Vec<Note> = notes.to_vec();
            down.reverse();
            result.extend(down.into_iter().skip(1));
            result
        }
        ArpPattern::DownUp => {
            let mut result = notes.to_vec();
            result.reverse();
            result.extend(notes.iter().skip(1).cloned());
            result
        }
        ArpPattern::Random => {
            let mut result = notes.to_vec();
            result.shuffle(rng);
            result
        }
        ArpPattern::Converge => {
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
        ArpPattern::Diverge => {
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
        ArpPattern::PingPong => {
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
        ArpPattern::Thirds => {
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
        ArpPattern::Skip => {
            let mut result = Vec::new();
            for i in (0..n).step_by(2) {
                result.push(notes[i].clone());
            }
            for i in (1..n).step_by(2) {
                result.push(notes[i].clone());
            }
            result
        }
        ArpPattern::Walk => {
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
        ArpPattern::Pairs => {
            let mut result = Vec::new();
            for chunk in notes.chunks(2) {
                result.extend(chunk.iter().cloned());
                result.extend(chunk.iter().cloned());
            }
            result
        }
        ArpPattern::Triplets => {
            let mut result = Vec::new();
            for i in 0..n.saturating_sub(2) {
                result.push(notes[i].clone());
                result.push(notes[i + 1].clone());
                result.push(notes[i + 2].clone());
            }
            if n == 1 {
                result.push(notes[0].clone());
            } else if n == 2 {
                result.push(notes[0].clone());
                result.push(notes[1].clone());
            }
            result
        }
        ArpPattern::Spiral => {
            let converged = apply_pattern(rng, notes, ArpPattern::Converge);
            let mut result = converged.clone();
            let mut diverged = converged;
            diverged.reverse();
            result.extend(diverged.into_iter().skip(1));
            result
        }
        ArpPattern::Leaps => {
            let mut result = Vec::new();
            for start in 0..3 {
                for i in (start..n).step_by(3) {
                    result.push(notes[i].clone());
                }
            }
            result
        }
        ArpPattern::Zipper => {
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

/// A step with resolved note (or rest indication)
struct ResolvedStep {
    note: Option<Note>,
    duration_beats: f32,
    velocity: f32,
}

/// Variation parameters for a section
#[derive(Debug, Clone)]
struct SectionVariation {
    /// How many groove loops to play
    loops: u32,
    /// Chance of dropping a note (0.0 - 1.0)
    drop_chance: f32,
    /// Chance of octave shift (0.0 - 1.0)
    octave_shift_chance: f32,
    /// Velocity humanization amount (random wobble)
    velocity_wobble: f32,
    /// Dynamic shape: "wave", "crescendo", "decrescendo", "steady"
    dynamic_shape: DynamicShape,
    /// Chance of double-time (splitting a note into two fast notes)
    double_time_chance: f32,
    /// Chance of playing a whole loop in double-time
    double_time_loop_chance: f32,
}

#[derive(Debug, Clone, Copy)]
enum DynamicShape {
    Steady,
    Wave,
    Crescendo,
    Decrescendo,
}

impl SectionVariation {
    /// Create a random variation with fill controlling note density
    /// fill: 0.0 = sparse (minimal double-time), 1.0 = dense (maximum double-time)
    fn random<R: Rng>(rng: &mut R, fill: f32) -> Self {
        let fill = fill.clamp(0.0, 1.0);
        // Scale double-time chances based on fill
        // At fill=0: double_time_chance ~0.02, loop_chance ~0.05
        // At fill=1: double_time_chance ~0.40, loop_chance ~0.50
        let base_double_time = 0.02 + fill * 0.38;
        let base_loop_double = 0.05 + fill * 0.45;
        Self {
            loops: rng.gen_range(12..=20),
            drop_chance: rng.gen_range(0.02..0.10),
            octave_shift_chance: rng.gen_range(0.05..0.12),
            velocity_wobble: rng.gen_range(0.05..0.12),
            dynamic_shape: match rng.gen_range(0..4) {
                0 => DynamicShape::Steady,
                1 => DynamicShape::Wave,
                2 => DynamicShape::Crescendo,
                _ => DynamicShape::Decrescendo,
            },
            double_time_chance: base_double_time + rng.gen_range(-0.05..0.05),
            double_time_loop_chance: base_loop_double + rng.gen_range(-0.05..0.05),
        }
    }

    fn for_single_mode(loops: u32, fill: f32) -> Self {
        let fill = fill.clamp(0.0, 1.0);
        Self {
            loops,
            drop_chance: 0.05,
            octave_shift_chance: 0.08,
            velocity_wobble: 0.1,
            dynamic_shape: DynamicShape::Wave,
            double_time_chance: 0.02 + fill * 0.38,
            double_time_loop_chance: 0.05 + fill * 0.45,
        }
    }

    /// Variation for export - keeps most variations but ensures predictable note count
    fn for_export<R: Rng>(rng: &mut R, loops: u32, fill: f32) -> Self {
        let fill = fill.clamp(0.0, 1.0);
        Self {
            loops,
            drop_chance: 0.0,           // No dropped notes (changes note count)
            octave_shift_chance: rng.gen_range(0.05..0.12),
            velocity_wobble: rng.gen_range(0.05..0.12),
            dynamic_shape: match rng.gen_range(0..4) {
                0 => DynamicShape::Steady,
                1 => DynamicShape::Wave,
                2 => DynamicShape::Crescendo,
                _ => DynamicShape::Decrescendo,
            },
            double_time_chance: 0.0,    // No note-level double-time (adds extra notes)
            double_time_loop_chance: 0.05 + fill * 0.45, // Scale with fill
        }
    }
}

/// Calculate dynamic multiplier based on position in section
fn dynamic_multiplier(shape: DynamicShape, progress: f32) -> f32 {
    match shape {
        DynamicShape::Steady => 1.0,
        DynamicShape::Wave => {
            // Two waves per section, subtle variation
            let wave = (progress * 2.0 * std::f32::consts::PI * 2.0).sin();
            0.85 + 0.15 * wave
        }
        DynamicShape::Crescendo => 0.75 + 0.25 * progress,
        DynamicShape::Decrescendo => 1.0 - 0.2 * progress,
    }
}

/// Shift a note by octaves
fn shift_octave(note: &Note, shift: i8) -> Option<Note> {
    let midi = note.midi_value() as i16 + (shift as i16 * 12);
    if midi >= 24 && midi <= 108 {
        midi_to_note(midi as u8).ok()
    } else {
        Some(note.clone())
    }
}

/// Apply a groove to a sequence of notes with variation
fn apply_groove_with_variation<R: Rng>(
    rng: &mut R,
    notes: &[Note],
    groove: &Groove,
    variation: &SectionVariation,
) -> Vec<ResolvedStep> {
    let mut result = Vec::new();
    let mut note_idx = 0;

    let total_steps = variation.loops as usize * groove.steps.len();

    for loop_num in 0..variation.loops {
        // Check if this entire loop should be double-time
        let loop_double_time = rng.gen::<f32>() < variation.double_time_loop_chance;
        let time_scale = if loop_double_time { 0.5 } else { 1.0 };

        for (step_in_loop, step) in groove.steps.iter().enumerate() {
            let global_step = loop_num as usize * groove.steps.len() + step_in_loop;
            let progress = global_step as f32 / total_steps as f32;

            // Apply time scale and enforce minimum duration
            let step_beats = (step.beats * time_scale).max(MIN_NOTE_BEATS);

            if step.is_rest {
                result.push(ResolvedStep {
                    note: None,
                    duration_beats: step_beats,
                    velocity: 0.0,
                });
            } else {
                // Maybe drop this note
                if rng.gen::<f32>() < variation.drop_chance {
                    result.push(ResolvedStep {
                        note: None,
                        duration_beats: step_beats,
                        velocity: 0.0,
                    });
                    note_idx += 1;
                    continue;
                }

                // Apply dynamics and humanization
                let dyn_mult = dynamic_multiplier(variation.dynamic_shape, progress);
                let wobble = 1.0 + rng.gen_range(-variation.velocity_wobble..variation.velocity_wobble);
                let velocity = (step.velocity * dyn_mult * wobble).clamp(0.5, 1.0);

                // Check for note-level double-time (split into two notes)
                // Only if half duration would still be >= minimum
                let half_duration = step_beats * 0.5;
                let can_double_time = !loop_double_time && half_duration >= MIN_NOTE_BEATS;
                let note_double_time = can_double_time && rng.gen::<f32>() < variation.double_time_chance;

                if note_double_time {
                    // Split into two fast notes
                    // First note
                    let base_note1 = &notes[note_idx % notes.len()];
                    let note1 = if rng.gen::<f32>() < variation.octave_shift_chance {
                        let shift = if rng.gen_bool(0.5) { 1 } else { -1 };
                        shift_octave(base_note1, shift).unwrap_or_else(|| base_note1.clone())
                    } else {
                        base_note1.clone()
                    };
                    result.push(ResolvedStep {
                        note: Some(note1),
                        duration_beats: half_duration,
                        velocity,
                    });
                    note_idx += 1;

                    // Second note (next in sequence)
                    let base_note2 = &notes[note_idx % notes.len()];
                    let note2 = if rng.gen::<f32>() < variation.octave_shift_chance {
                        let shift = if rng.gen_bool(0.5) { 1 } else { -1 };
                        shift_octave(base_note2, shift).unwrap_or_else(|| base_note2.clone())
                    } else {
                        base_note2.clone()
                    };
                    // Second note slightly softer
                    let velocity2 = (velocity * 0.85).clamp(0.5, 1.0);
                    result.push(ResolvedStep {
                        note: Some(note2),
                        duration_beats: half_duration,
                        velocity: velocity2,
                    });
                    note_idx += 1;
                } else {
                    // Normal single note
                    let base_note = &notes[note_idx % notes.len()];
                    let note = if rng.gen::<f32>() < variation.octave_shift_chance {
                        let shift = if rng.gen_bool(0.5) { 1 } else { -1 };
                        shift_octave(base_note, shift).unwrap_or_else(|| base_note.clone())
                    } else {
                        base_note.clone()
                    };

                    result.push(ResolvedStep {
                        note: Some(note),
                        duration_beats: step_beats,
                        velocity,
                    });
                    note_idx += 1;
                }
            }
        }
    }

    result
}

/// Add resolved steps to a sequence, returning the end position
fn add_steps_to_sequence(
    seq: &mut Sequence,
    steps: &[ResolvedStep],
    start_ticks: u32,
) -> u32 {
    let time_sig = seq.time_signature();
    let release = 0.85;
    let mut current_ticks = start_ticks;

    for step in steps {
        let duration = time_sig.beat_time(step.duration_beats);

        if let Some(ref note) = step.note {
            let time = Time { ticks: current_ticks };
            let note_duration = duration * release;
            seq.add_note(time, note.clone(), step.velocity, note_duration);
        }
        // Always advance time (rests advance time without adding notes)
        current_ticks += duration.ticks;
    }

    current_ticks
}

/// Add alternating chord progression (root triad <-> augmented) over the sequence
/// with optional strum effect (notes played with slight time offsets)
fn add_chord_progression<R: Rng>(
    seq: &mut Sequence,
    rng: &mut R,
    root: &Note,
    scale_notes: &[u8],
    start_ticks: u32,
    end_ticks: u32,
    chord_duration_beats: f32,
    velocity: f32,
    strum_ticks: u32,
) {
    let time_sig = seq.time_signature();
    let chord_duration = time_sig.beat_time(chord_duration_beats);
    let release = 0.9;

    // Build the two alternating chords
    // Use root an octave lower for fuller sound
    let bass_root = shift_octave(root, -1).unwrap_or_else(|| root.clone());
    let root_triad = build_scale_triad(&bass_root, scale_notes);
    let aug_chord = build_augmented_chord(&bass_root);

    let mut current_ticks = start_ticks;
    let mut use_root = true;

    while current_ticks < end_ticks {
        let remaining = end_ticks - current_ticks;
        let actual_duration_ticks = chord_duration.ticks.min(remaining);

        let chord_notes = if use_root { &root_triad } else { &aug_chord };

        // Add each note with strum offset (humanized variance)
        for (i, note) in chord_notes.iter().enumerate() {
            // Calculate strum offset with humanized variance
            let base_offset = (i as u32) * strum_ticks;
            let humanized_offset = if strum_ticks > 0 {
                // Add variance of ±30% around the base offset
                let variance = (strum_ticks as f32 * 0.3) as i32;
                let jitter = if variance > 0 {
                    rng.gen_range(-variance..=variance)
                } else {
                    0
                };
                (base_offset as i32 + jitter).max(0) as u32
            } else {
                0
            };

            let note_time = Time { ticks: current_ticks + humanized_offset };
            // Shorten duration slightly to account for strum offset
            let note_duration = if humanized_offset < actual_duration_ticks {
                Time { ticks: (actual_duration_ticks - humanized_offset) }
            } else {
                Time { ticks: 1 } // Minimum duration
            };

            seq.add_note_on_channel(
                note_time,
                note.clone(),
                velocity,
                note_duration * release,
                1, // bass channel
            );
        }

        current_ticks += chord_duration.ticks;
        use_root = !use_root;
    }
}

fn get_scale_notes(scale_name: &str) -> Result<Vec<u8>, String> {
    let scale = get_scale(scale_name).map_err(|e| e.to_string())?;

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
                eprintln!("Usage: groove_improv [OPTIONS] [SOUNDFONT]");
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

/// State for a single playable variation
#[derive(Clone)]
struct PlayState {
    seed: u64,
    variation_index: u32,
    scale_name: String,
    root: Note,
    pattern: ArpPattern,
    groove: Groove,
}

impl PlayState {
    /// Generate a new state from a seed and variation index
    fn from_seed(seed: u64, variation_index: u32, grooves: &[Groove]) -> Option<Self> {
        // Use seed to deterministically generate scale and root
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let scale_name = random_scale_name(&mut rng);
        let root = random_root_note(&mut rng);

        // Verify scale has notes
        if get_scale_notes(&scale_name).is_err() {
            return None;
        }

        // Use variation_index to pick pattern and groove deterministically
        let mut var_rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(variation_index as u64 * 1000));
        let pattern = random_pattern(&mut var_rng);
        let groove = random_groove(&mut var_rng, grooves);

        Some(Self {
            seed,
            variation_index,
            scale_name,
            root,
            pattern,
            groove,
        })
    }

    /// Build and return the sequence for this state
    fn build_sequence(&self, octaves: u32, strum_ticks: u32, fill: f32) -> Result<Sequence, Box<dyn std::error::Error>> {
        self.build_sequence_with_options(octaves, strum_ticks, fill, false)
    }

    /// Build sequence with export options
    /// If export is true, calculates loops for perfect alignment
    fn build_sequence_with_options(
        &self,
        octaves: u32,
        strum_ticks: u32,
        fill: f32,
        export: bool,
    ) -> Result<Sequence, Box<dyn std::error::Error>> {
        let scale_notes = get_scale_notes(&self.scale_name)?;
        let scale = get_scale(&self.scale_name)?;

        let time_signature = common_time();
        let mut seq = Sequence::new(&format!("{} Improv", scale.name), time_signature)?;

        // Use deterministic RNG for the entire build
        let var_seed = self.seed.wrapping_add(self.variation_index as u64 * 1000 + 500);
        let mut rng = ChaCha8Rng::seed_from_u64(var_seed);

        let base_notes = build_base_notes(&self.root, &scale_notes, octaves);
        let pattern_notes = apply_pattern(&mut rng, &base_notes, self.pattern);

        let variation = if export {
            // Count non-rest steps in groove (these consume notes)
            let note_steps_per_loop = self.groove.steps.iter().filter(|s| !s.is_rest).count();
            // Calculate loops needed for pattern to align perfectly
            let aligned_loops = loops_for_alignment(note_steps_per_loop, pattern_notes.len());
            // Use at least 1 loop, cap at reasonable maximum
            let loops = aligned_loops.clamp(1, 32) as u32;
            SectionVariation::for_export(&mut rng, loops, fill)
        } else {
            SectionVariation::random(&mut rng, fill)
        };
        let forward_steps = apply_groove_with_variation(&mut rng, &pattern_notes, &self.groove, &variation);

        // Add forward steps
        let midpoint = add_steps_to_sequence(&mut seq, &forward_steps, 0);

        // Mirror: reverse the notes but keep the rhythm (durations/velocities) forward
        // Collect just the notes in reverse order
        let reversed_notes: Vec<Option<Note>> = forward_steps.iter()
            .map(|s| s.note.clone())
            .rev()
            .skip(1)  // Skip last note (same as first of forward when looping)
            .collect();

        // Apply reversed notes to forward rhythm, skipping last step (same as first)
        let steps_to_use = forward_steps.len().saturating_sub(1);
        let reversed_steps: Vec<ResolvedStep> = forward_steps.iter()
            .take(steps_to_use)
            .zip(reversed_notes.iter())
            .map(|(step, rev_note)| ResolvedStep {
                note: rev_note.clone(),
                duration_beats: step.duration_beats,
                velocity: step.velocity,
            })
            .collect();

        let end_ticks = add_steps_to_sequence(&mut seq, &reversed_steps, midpoint);

        // Add alternating chord progression (root triad <-> augmented)
        // Chords change every 2 beats for harmonic movement
        add_chord_progression(
            &mut seq,
            &mut rng,
            &self.root,
            &scale_notes,
            0,
            end_ticks,
            2.0,  // chord changes every 2 beats
            0.5,  // softer than melody
            strum_ticks,
        );

        Ok(seq)
    }

    fn display_with_instruments(&self, lead: Option<u8>, bass: Option<u8>, mute_lead: bool, mute_bass: bool) {
        let scale = get_scale(&self.scale_name).unwrap();

        // Build the command line args
        let mut args = format!("--seed {}", self.seed);
        if self.variation_index != 0 {
            args.push_str(&format!(" --variation {}", self.variation_index));
        }
        if let Some(prog) = lead {
            args.push_str(&format!(" --lead {}", prog));
        }
        if let Some(prog) = bass {
            args.push_str(&format!(" --bass {}", prog));
        }
        if mute_lead {
            args.push_str(" --ml");
        }
        if mute_bass {
            args.push_str(" --mb");
        }
        println!("\n{}", args);

        // Show instrument names and mute status
        let lead_name = lead.map(|p| format!("{} ({})", p, gm_instrument_name(p)))
            .unwrap_or_else(|| "default".to_string());
        let bass_name = bass.map(|p| format!("{} ({})", p, gm_instrument_name(p)))
            .unwrap_or_else(|| "33 (Electric Bass (finger))".to_string());
        let lead_status = if mute_lead { " [MUTED]" } else { "" };
        let bass_status = if mute_bass { " [MUTED]" } else { "" };
        println!("Lead: {}{} | Bass: {}{}", lead_name, lead_status, bass_name, bass_status);

        println!(
            "Scale: {} | Root: {} | Pattern: {:?} | Groove: {}",
            scale.name,
            note_name(&self.root),
            self.pattern,
            self.groove.name
        );
    }
}

/// Spawn a background thread that reads stdin with readline support and sends commands through a channel
fn spawn_input_thread() -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut rl = match DefaultEditor::new() {
            Ok(editor) => editor,
            Err(e) => {
                eprintln!("Failed to initialize readline: {}", e);
                return;
            }
        };
        loop {
            match rl.readline("> ") {
                Ok(line) => {
                    let cmd = line.trim().to_lowercase();
                    // Add non-empty commands to history
                    if !cmd.is_empty() {
                        let _ = rl.add_history_entry(&line);
                    }
                    if tx.send(cmd).is_err() {
                        break; // Receiver dropped, exit thread
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C - treat as quit
                    let _ = tx.send("q".to_string());
                    break;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D - treat as quit
                    let _ = tx.send("q".to_string());
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }
    });
    rx
}

/// Audio playback loop that runs in background, checking stop flag during playback
fn spawn_audio_loop(
    player: Arc<Player>,
    midi_bytes: Vec<u8>,
    stop_flag: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !stop_flag.load(Ordering::Relaxed) {
            // Use stoppable version that checks flag during playback
            match player.play_midi_bytes_stoppable(&midi_bytes, &stop_flag) {
                Ok(false) => break, // Stopped early
                Err(_) => break,    // Error
                Ok(true) => {}      // Completed normally, loop again
            }
        }
    })
}

fn run_demo_mode(
    soundfont: SoundFontSource,
    octaves: u32,
    bpm: u32,
    strum_ticks: u32,
    fill: f32,
    initial_seed: Option<u64>,
    initial_variation: u32,
    initial_lead: Option<u8>,
    initial_bass: Option<u8>,
    initial_mute_lead: bool,
    initial_mute_bass: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Using SoundFont: {}", soundfont.path().display());
    println!("\n=== IMPROV DEMO MODE ===");
    println!("Loops continuously. Commands are processed immediately.");
    println!("Commands: (Enter)=new | j/k=next/prev | g/p=groove/pattern | il/ib=instr | ml/mb=mute | b=bpm | #=seed | q=quit");
    println!("BPM: {}\n", bpm);

    // Make bpm mutable for runtime changes
    let mut bpm = bpm;

    let player = Arc::new(Player::new(soundfont)?);
    let grooves = all_grooves();

    // Start background input thread
    let input_rx = spawn_input_thread();

    // History stack for going back
    let mut history: Vec<PlayState> = Vec::new();

    // Generate initial seed
    let mut current_seed = initial_seed.unwrap_or_else(|| rand::thread_rng().gen());
    let mut variation_index = initial_variation;

    // Current GM instruments
    let mut current_lead: Option<u8> = initial_lead;
    let mut current_bass: Option<u8> = initial_bass;

    // Mute states
    let mut mute_lead = initial_mute_lead;
    let mut mute_bass = initial_mute_bass;

    // Override groove/pattern (None = use seed-derived values)
    let mut groove_override: Option<usize> = None;
    let mut pattern_override: Option<usize> = None;

    // Current audio thread handle and stop flag
    let mut audio_handle: Option<thread::JoinHandle<()>> = None;
    let mut stop_flag = Arc::new(AtomicBool::new(false));

    // Start with first state
    let mut need_new_audio = true;

    loop {
        if need_new_audio {
            // Stop any existing audio and wait for it
            stop_flag.store(true, Ordering::Relaxed);
            if let Some(handle) = audio_handle.take() {
                print!("  Stopping... ");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                let _ = handle.join();
                println!("done");
                // Brief silence to clear the ears
                thread::sleep(std::time::Duration::from_millis(400));
            }

            // Find a valid state (some scales don't have notes)
            let mut state = loop {
                if let Some(s) = PlayState::from_seed(current_seed, variation_index, &grooves) {
                    break s;
                }
                current_seed = current_seed.wrapping_add(1);
            };

            // Apply overrides
            if let Some(idx) = groove_override {
                state.groove = grooves[idx % grooves.len()].clone();
            }
            if let Some(idx) = pattern_override {
                state.pattern = ALL_PATTERNS[idx % ALL_PATTERNS.len()];
            }

            state.display_with_instruments(current_lead, current_bass, mute_lead, mute_bass);
            println!("  (looping - Enter=new, j/k=next/prev, g/p=groove/pattern, il/ib, ml/mb, b=bpm, #=seed, q=quit)");

            // Build the sequence and convert to MIDI bytes with tempo and instruments
            let seq = state.build_sequence(octaves, strum_ticks, fill)?;
            let smf = seq.to_midi();
            let mut midi_buffer = Vec::new();
            smf.write_std(&mut midi_buffer)?;
            let midi_bytes = prepare_midi_bytes(&midi_buffer, bpm, current_lead, current_bass, mute_lead, mute_bass);

            // Start new audio loop
            stop_flag = Arc::new(AtomicBool::new(false));
            audio_handle = Some(spawn_audio_loop(
                Arc::clone(&player),
                midi_bytes,
                Arc::clone(&stop_flag),
            ));

            // Store state for history
            history.push(PlayState::from_seed(current_seed, variation_index, &grooves).unwrap());
            need_new_audio = false;
        }

        // Wait for a command (blocking)
        let cmd = match input_rx.recv() {
            Ok(cmd) => cmd,
            Err(_) => "q".to_string(),
        };

        // Process command immediately
        match cmd.as_str() {
            "" => {
                // Enter: new scale (new seed)
                println!("\n  -> New scale...");
                current_seed = rand::thread_rng().gen();
                variation_index = 0;
                need_new_audio = true;
            }
            "j" | "next" => {
                // Next variation of same scale
                println!("\n  -> Next variation...");
                variation_index += 1;
                need_new_audio = true;
            }
            "k" | "prev" | "back" => {
                // Go back
                if history.len() > 1 {
                    history.pop(); // Remove current
                    if let Some(prev) = history.last() {
                        println!("\n  -> Going back...");
                        current_seed = prev.seed;
                        variation_index = prev.variation_index;
                        need_new_audio = true;
                    }
                } else {
                    println!("  (no history to go back to)");
                }
            }
            "ml" => {
                // Toggle lead mute
                mute_lead = !mute_lead;
                println!("\n  -> Lead: {}", if mute_lead { "MUTED" } else { "unmuted" });
                need_new_audio = true;
            }
            "mb" => {
                // Toggle bass mute
                mute_bass = !mute_bass;
                println!("\n  -> Bass: {}", if mute_bass { "MUTED" } else { "unmuted" });
                need_new_audio = true;
            }
            "s" | "stop" => {
                // Stop playback but keep waiting for input
                println!("\n  -> Stopping playback...");
                stop_flag.store(true, Ordering::Relaxed);
                if let Some(handle) = audio_handle.take() {
                    let _ = handle.join();
                }
                println!("  Stopped. Enter a command to resume.");
            }
            "q" | "quit" | "exit" => {
                println!("\nExiting...");
                stop_flag.store(true, Ordering::Relaxed);
                return Ok(());
            }
            _ => {
                // Check for b (BPM) command
                if cmd == "b" || cmd.starts_with("b ") || cmd.starts_with("b") && cmd.len() > 1 && cmd[1..].chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    let num_str = cmd.strip_prefix("b").unwrap().trim();
                    let new_bpm: u32 = if num_str.is_empty() {
                        rand::thread_rng().gen_range(30..=200)
                    } else if let Ok(n) = num_str.parse::<u32>() {
                        if n < 30 || n > 200 {
                            println!("  BPM must be 30-200");
                            continue;
                        }
                        n
                    } else {
                        println!("  Invalid BPM");
                        continue;
                    };
                    bpm = new_bpm;
                    println!("\n  -> BPM: {}", bpm);
                    need_new_audio = true;
                }
                // Check for g (groove) command
                else if cmd == "g" || cmd.starts_with("g ") || cmd.starts_with("g") && cmd.len() > 1 && cmd[1..].chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    let num_str = cmd.strip_prefix("g").unwrap().trim();
                    let idx: usize = if num_str.is_empty() {
                        rand::thread_rng().gen_range(0..grooves.len())
                    } else if let Ok(n) = num_str.parse::<usize>() {
                        if n >= grooves.len() {
                            println!("  Groove must be 0-{}", grooves.len() - 1);
                            continue;
                        }
                        n
                    } else {
                        println!("  Invalid groove index");
                        continue;
                    };
                    groove_override = Some(idx);
                    println!("\n  -> Groove: {} ({})", idx, grooves[idx].name);
                    need_new_audio = true;
                }
                // Check for p (pattern) command
                else if cmd == "p" || cmd.starts_with("p ") || cmd.starts_with("p") && cmd.len() > 1 && cmd[1..].chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    let num_str = cmd.strip_prefix("p").unwrap().trim();
                    let idx: usize = if num_str.is_empty() {
                        rand::thread_rng().gen_range(0..ALL_PATTERNS.len())
                    } else if let Ok(n) = num_str.parse::<usize>() {
                        if n >= ALL_PATTERNS.len() {
                            println!("  Pattern must be 0-{}", ALL_PATTERNS.len() - 1);
                            continue;
                        }
                        n
                    } else {
                        println!("  Invalid pattern index");
                        continue;
                    };
                    pattern_override = Some(idx);
                    println!("\n  -> Pattern: {} ({:?})", idx, ALL_PATTERNS[idx]);
                    need_new_audio = true;
                }
                // Check for il (lead instrument) command
                else if cmd == "il" || cmd.starts_with("il ") || cmd.starts_with("il") && cmd.len() > 2 && cmd[2..].chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    let num_str = cmd.strip_prefix("il").unwrap().trim();
                    let prog: u8 = if num_str.is_empty() {
                        rand::thread_rng().gen_range(0..128)
                    } else if let Ok(n) = num_str.parse::<u8>() {
                        if n > 127 {
                            println!("  Instrument must be 0-127");
                            continue;
                        }
                        n
                    } else {
                        println!("  Invalid instrument number");
                        continue;
                    };
                    current_lead = Some(prog);
                    println!("\n  -> Lead: {} ({})", prog, gm_instrument_name(prog));
                    need_new_audio = true;
                }
                // Check for ib (bass instrument) command
                else if cmd == "ib" || cmd.starts_with("ib ") || cmd.starts_with("ib") && cmd.len() > 2 && cmd[2..].chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    let num_str = cmd.strip_prefix("ib").unwrap().trim();
                    let prog: u8 = if num_str.is_empty() {
                        rand::thread_rng().gen_range(0..128)
                    } else if let Ok(n) = num_str.parse::<u8>() {
                        if n > 127 {
                            println!("  Instrument must be 0-127");
                            continue;
                        }
                        n
                    } else {
                        println!("  Invalid instrument number");
                        continue;
                    };
                    current_bass = Some(prog);
                    println!("\n  -> Bass: {} ({})", prog, gm_instrument_name(prog));
                    need_new_audio = true;
                }
                // Try to parse as a seed number
                else if let Ok(new_seed) = cmd.parse::<u64>() {
                    println!("\n  -> Jumping to seed {}...", new_seed);
                    current_seed = new_seed;
                    variation_index = 0;
                    need_new_audio = true;
                } else {
                    println!("  Unknown command '{}'. Use Enter, j, k, il, ib, b, ml, mb, s, q, or a seed", cmd);
                }
            }
        }
    }
}

fn run_single_mode(
    soundfont: SoundFontSource,
    scale_name: &str,
    root: Note,
    pattern: ArpPattern,
    groove: &Groove,
    loops: u32,
    octaves: u32,
    bpm: u32,
    fill: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Using SoundFont: {}", soundfont.path().display());
    println!("Root note: {} (MIDI {})", note_name(&root), root.midi_value());

    let scale_notes = get_scale_notes(scale_name)?;
    let scale = get_scale(scale_name)?;
    println!("Scale: {} - notes: {:?}", scale.name, scale_notes);
    println!(
        "Pattern: {:?} | Groove: {} | Loops: {} | Octaves: {} | BPM: {}",
        pattern, groove.name, loops, octaves, bpm
    );

    let time_signature = common_time();
    let mut seq = Sequence::new(&format!("{} Improv", scale.name), time_signature)?;

    let mut rng = rand::thread_rng();
    let base_notes = build_base_notes(&root, &scale_notes, octaves);
    let pattern_notes = apply_pattern(&mut rng, &base_notes, pattern);
    let variation = SectionVariation::for_single_mode(loops, fill);
    let steps = apply_groove_with_variation(&mut rng, &pattern_notes, groove, &variation);
    add_steps_to_sequence(&mut seq, &steps, 0);

    let player = Player::new(soundfont)?;
    play_sequence_with_bpm(&player, &seq, bpm)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set log level to warn to reduce noise
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "warn");
    }
    setup_log();

    let cli = Cli::parse();

    if cli.list_grooves {
        println!("Available grooves:");
        for groove in all_grooves() {
            println!("  {}", groove.name);
        }
        return Ok(());
    }

    // Export MIDI to file
    if let Some(ref output_path) = cli.output {
        let seed = cli.seed.ok_or("--output requires --seed to be specified")?;
        let grooves = all_grooves();

        let state = PlayState::from_seed(seed, cli.variation, &grooves)
            .ok_or_else(|| format!("Could not generate state for seed {}", seed))?;

        state.display_with_instruments(cli.lead, cli.bass, false, false);
        // Use clean export variation for perfect looping
        let seq = state.build_sequence_with_options(cli.octaves, cli.strum, cli.fill, true)?;
        let smf = seq.to_midi();
        let mut midi_buffer = Vec::new();
        smf.write_std(&mut midi_buffer)?;
        let midi_bytes = prepare_midi_bytes(&midi_buffer, cli.bpm, cli.lead, cli.bass, false, false);

        let mut file = File::create(output_path)?;
        file.write_all(&midi_bytes)?;
        println!("Exported to: {}", output_path);
        return Ok(());
    }

    let soundfont = load_soundfont(cli.soundfont.as_ref())?;
    let grooves = all_grooves();

    let demo_mode =
        cli.scale.is_none() && cli.root.is_none() && cli.pattern.is_none() && cli.groove.is_none();

    if demo_mode || cli.seed.is_some() || cli.variation > 0 || cli.lead.is_some() || cli.bass.is_some() {
        run_demo_mode(soundfont, cli.octaves, cli.bpm, cli.strum, cli.fill, cli.seed, cli.variation, cli.lead, cli.bass, cli.ml, cli.mb)?;
    } else {
        let scale_name = cli.scale.unwrap_or_else(|| "major".to_string());
        let root = if let Some(ref r) = cli.root {
            parse_root_note(r)?
        } else {
            parse_root_note("C4")?
        };
        let pattern = cli.pattern.unwrap_or(ArpPattern::Up);
        let groove = if let Some(ref g) = cli.groove {
            grooves
                .iter()
                .find(|gr| gr.name.eq_ignore_ascii_case(g))
                .cloned()
                .ok_or_else(|| format!("Unknown groove '{}'. Use --list-grooves to see options.", g))?
        } else {
            grooves[0].clone() // Default to first groove
        };

        run_single_mode(
            soundfont,
            &scale_name,
            root,
            pattern,
            &groove,
            cli.loops,
            cli.octaves,
            cli.bpm,
            cli.fill,
        )?;
    }

    Ok(())
}
