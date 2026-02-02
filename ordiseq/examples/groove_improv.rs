use clap::{Parser, ValueEnum};
use midly::{MetaMessage, Smf, TrackEvent, TrackEventKind};
use ordiseq::midi::HasMidiValue;
use ordiseq::prelude::*;
use ordiseq::synth::{SoundFontSource, PREFERRED_SOUNDFONTS};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use scale_omnibus::{get_scale, get_scale_names};

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
#[command(about = "Improvisational arpeggio player with rhythmic grooves. Run with no args for infinite demo mode.")]
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

fn random_root_note() -> Note {
    let mut rng = thread_rng();
    let midi: u8 = rng.gen_range(48..=72);
    midi_to_note(midi).expect("Invalid random MIDI note")
}

fn random_scale_name() -> String {
    let mut rng = thread_rng();
    let names = get_scale_names();
    names.choose(&mut rng).unwrap().clone()
}

fn random_pattern() -> ArpPattern {
    let mut rng = thread_rng();
    *ALL_PATTERNS.choose(&mut rng).unwrap()
}

fn random_groove(grooves: &[Groove]) -> Groove {
    let mut rng = thread_rng();
    grooves.choose(&mut rng).unwrap().clone()
}

fn note_name(note: &Note) -> String {
    format!("{:?}{}", note.named_pitch(), note.octave() as u8)
}

fn bpm_to_tempo(bpm: u32) -> u32 {
    60_000_000 / bpm
}

fn add_tempo_to_midi_bytes(midi_bytes: &[u8], bpm: u32) -> Vec<u8> {
    let tempo_us = bpm_to_tempo(bpm);

    let smf = Smf::parse(midi_bytes).expect("Failed to parse MIDI");
    let smf_owned = smf.make_static();

    let mut new_tracks: Vec<Vec<TrackEvent<'static>>> = Vec::new();

    for track in smf_owned.tracks {
        let mut new_track: Vec<TrackEvent<'static>> = Vec::new();
        let mut tempo_added = false;

        for event in track {
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
fn apply_pattern(notes: &[Note], pattern: ArpPattern) -> Vec<Note> {
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
            result.shuffle(&mut thread_rng());
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
            let converged = apply_pattern(notes, ArpPattern::Converge);
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
}

#[derive(Debug, Clone, Copy)]
enum DynamicShape {
    Steady,
    Wave,
    Crescendo,
    Decrescendo,
}

impl SectionVariation {
    fn random() -> Self {
        let mut rng = thread_rng();
        Self {
            loops: rng.gen_range(12..=20),
            drop_chance: rng.gen_range(0.02..0.12),
            octave_shift_chance: rng.gen_range(0.05..0.15),
            velocity_wobble: rng.gen_range(0.05..0.15),
            dynamic_shape: match rng.gen_range(0..4) {
                0 => DynamicShape::Steady,
                1 => DynamicShape::Wave,
                2 => DynamicShape::Crescendo,
                _ => DynamicShape::Decrescendo,
            },
        }
    }

    fn for_single_mode(loops: u32) -> Self {
        Self {
            loops,
            drop_chance: 0.05,
            octave_shift_chance: 0.08,
            velocity_wobble: 0.1,
            dynamic_shape: DynamicShape::Wave,
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
fn apply_groove_with_variation(
    notes: &[Note],
    groove: &Groove,
    variation: &SectionVariation,
) -> Vec<ResolvedStep> {
    let mut rng = thread_rng();
    let mut result = Vec::new();
    let mut note_idx = 0;

    let total_steps = variation.loops as usize * groove.steps.len();

    for loop_num in 0..variation.loops {
        for (step_in_loop, step) in groove.steps.iter().enumerate() {
            let global_step = loop_num as usize * groove.steps.len() + step_in_loop;
            let progress = global_step as f32 / total_steps as f32;

            if step.is_rest {
                result.push(ResolvedStep {
                    note: None,
                    duration_beats: step.beats,
                    velocity: 0.0,
                });
            } else {
                // Maybe drop this note
                if rng.gen::<f32>() < variation.drop_chance {
                    result.push(ResolvedStep {
                        note: None,
                        duration_beats: step.beats,
                        velocity: 0.0,
                    });
                    note_idx += 1;
                    continue;
                }

                let base_note = &notes[note_idx % notes.len()];

                // Maybe shift octave
                let note = if rng.gen::<f32>() < variation.octave_shift_chance {
                    let shift = if rng.gen_bool(0.5) { 1 } else { -1 };
                    shift_octave(base_note, shift).unwrap_or_else(|| base_note.clone())
                } else {
                    base_note.clone()
                };

                // Apply dynamics and humanization
                let dyn_mult = dynamic_multiplier(variation.dynamic_shape, progress);
                let wobble = 1.0 + rng.gen_range(-variation.velocity_wobble..variation.velocity_wobble);
                let velocity = (step.velocity * dyn_mult * wobble).clamp(0.5, 1.0);

                result.push(ResolvedStep {
                    note: Some(note),
                    duration_beats: step.beats,
                    velocity,
                });
                note_idx += 1;
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

fn run_demo_mode(
    soundfont: SoundFontSource,
    octaves: u32,
    bpm: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Using SoundFont: {}", soundfont.path().display());
    println!("\n=== IMPROV DEMO MODE ===");
    println!("Playing random scales with random patterns and grooves.");
    println!("BPM: {} | Press Ctrl+C to stop.\n", bpm);

    let player = Player::new(soundfont)?;
    let grooves = all_grooves();

    loop {
        let scale_name = random_scale_name();
        let root = random_root_note();

        let scale_notes = match get_scale_notes(&scale_name) {
            Ok(notes) => notes,
            Err(_) => continue,
        };

        let scale = get_scale(&scale_name)?;

        // Pick 3 random pattern+groove combinations
        let combos: Vec<(ArpPattern, Groove)> = (0..3)
            .map(|_| (random_pattern(), random_groove(&grooves)))
            .collect();

        println!(
            "Scale: {} | Root: {} | Octaves: {}",
            scale.name,
            note_name(&root),
            octaves
        );
        for (i, (pattern, groove)) in combos.iter().enumerate() {
            println!("  Section {}: {:?} + {}", i + 1, pattern, groove.name);
        }

        let time_signature = common_time();
        let mut seq = Sequence::new(&format!("{} Improv", scale.name), time_signature)?;

        let base_notes = build_base_notes(&root, &scale_notes, octaves);
        let mut current_ticks = 0u32;

        for (pattern, groove) in &combos {
            let pattern_notes = apply_pattern(&base_notes, *pattern);
            let variation = SectionVariation::random();
            let steps = apply_groove_with_variation(&pattern_notes, groove, &variation);
            current_ticks = add_steps_to_sequence(&mut seq, &steps, current_ticks);
        }

        print!("  Playing... ");
        std::io::Write::flush(&mut std::io::stdout())?;

        play_sequence_with_bpm(&player, &seq, bpm)?;
        println!("done\n");
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

    let base_notes = build_base_notes(&root, &scale_notes, octaves);
    let pattern_notes = apply_pattern(&base_notes, pattern);
    let variation = SectionVariation::for_single_mode(loops);
    let steps = apply_groove_with_variation(&pattern_notes, groove, &variation);
    add_steps_to_sequence(&mut seq, &steps, 0);

    let player = Player::new(soundfont)?;
    play_sequence_with_bpm(&player, &seq, bpm)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let cli = Cli::parse();

    if cli.list_grooves {
        println!("Available grooves:");
        for groove in all_grooves() {
            println!("  {}", groove.name);
        }
        return Ok(());
    }

    let soundfont = load_soundfont(cli.soundfont.as_ref())?;
    let grooves = all_grooves();

    let demo_mode =
        cli.scale.is_none() && cli.root.is_none() && cli.pattern.is_none() && cli.groove.is_none();

    if demo_mode {
        run_demo_mode(soundfont, cli.octaves, cli.bpm)?;
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
        )?;
    }

    Ok(())
}
