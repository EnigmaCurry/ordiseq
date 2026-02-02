use clap::{Parser, ValueEnum};
use midly::{MetaMessage, Smf, TrackEvent, TrackEventKind};
use ordiseq::midi::HasMidiValue;
use ordiseq::prelude::*;
use ordiseq::synth::{SoundFontSource, PREFERRED_SOUNDFONTS};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scale_omnibus::{get_scale, get_scale_names};
use std::fs::File;
use std::io::Write;

// ============================================================================
// Arpeggio Patterns
// ============================================================================

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
enum ArpPattern {
    /// Ascending order
    Up,
    /// Descending order
    Down,
    /// Ascending then descending
    UpDown,
    /// Descending then ascending
    DownUp,
    /// Random order
    Random,
    /// Outside-in: alternates low/high moving inward
    Converge,
    /// Inside-out: starts from middle, expands outward
    Diverge,
    /// Bounces at ends
    PingPong,
    /// Broken thirds (1,3,2,4,3,5...)
    Thirds,
    /// Every other note, then fill gaps
    Skip,
    /// Two steps forward, one back
    Walk,
    /// Pairs of adjacent notes
    Pairs,
    /// Groups of 3 with overlap
    Triplets,
    /// Converge then diverge
    Spiral,
    /// Skip 2, then fill
    Leaps,
    /// Interleaved from both ends
    Zipper,
}

/// Apply an arpeggio pattern to reorder notes
fn apply_arp_pattern<R: Rng>(rng: &mut R, notes: &[Note], pattern: ArpPattern) -> Vec<Note> {
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
            result.extend(notes.iter().skip(1).copied());
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
                result.push(notes[lo]);
                if lo != hi {
                    result.push(notes[hi]);
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
            result.push(notes[mid]);
            for offset in 1..=mid {
                if mid + offset < n {
                    result.push(notes[mid + offset]);
                }
                if mid >= offset {
                    result.push(notes[mid - offset]);
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
                result.push(notes[i]);
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
                result.push(notes[i]);
                if i + 2 < n {
                    result.push(notes[i + 2]);
                }
                i += 1;
            }
            result
        }
        ArpPattern::Skip => {
            let mut result = Vec::new();
            for i in (0..n).step_by(2) {
                result.push(notes[i]);
            }
            for i in (1..n).step_by(2) {
                result.push(notes[i]);
            }
            result
        }
        ArpPattern::Walk => {
            let mut result = Vec::new();
            let mut i = 0;
            while i < n {
                result.push(notes[i]);
                if i + 1 < n {
                    result.push(notes[i + 1]);
                    result.push(notes[i]);
                }
                i += 1;
            }
            result
        }
        ArpPattern::Pairs => {
            let mut result = Vec::new();
            for chunk in notes.chunks(2) {
                result.extend(chunk.iter().copied());
                result.extend(chunk.iter().copied());
            }
            result
        }
        ArpPattern::Triplets => {
            let mut result = Vec::new();
            for i in 0..n.saturating_sub(2) {
                result.push(notes[i]);
                result.push(notes[i + 1]);
                result.push(notes[i + 2]);
            }
            if n == 1 {
                result.push(notes[0]);
            } else if n == 2 {
                result.push(notes[0]);
                result.push(notes[1]);
            }
            result
        }
        ArpPattern::Spiral => {
            let converged = apply_arp_pattern(rng, notes, ArpPattern::Converge);
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
                    result.push(notes[i]);
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
                    result.push(notes[lo]);
                    lo += 1;
                } else {
                    result.push(notes[hi]);
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

// ============================================================================
// Groove System
// ============================================================================

/// A rhythmic groove defines note durations, velocities, and rests
#[derive(Debug, Clone)]
struct Groove {
    name: &'static str,
    steps: Vec<GrooveStep>,
}

#[derive(Debug, Clone, Copy)]
struct GrooveStep {
    beats: f32,
    velocity: f32,
    is_rest: bool,
}

impl GrooveStep {
    fn note(beats: f32, velocity: f32) -> Self {
        Self { beats, velocity, is_rest: false }
    }
    fn rest(beats: f32) -> Self {
        Self { beats, velocity: 0.0, is_rest: true }
    }
}

fn all_grooves() -> Vec<Groove> {
    vec![
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

fn get_groove(name: &str) -> Option<Groove> {
    all_grooves().into_iter().find(|g| g.name.eq_ignore_ascii_case(name))
}

fn list_grooves() -> Vec<&'static str> {
    vec![
        "straight-quarters", "straight-eighths", "swing", "hard-swing",
        "syncopated", "offbeat", "dotted-quarters", "dotted-eighths",
        "triplets", "shuffle", "accented-twos", "accented-threes",
        "accented-fours", "sparse", "breathing", "clave", "bossa",
        "funk", "reggae", "driving",
    ]
}

/// Chord quality determined by interval pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Dom7,
    Maj7,
    Min7,
    HalfDim7,
    Dim7,
    MinMaj7,
    AugMaj7,
    Aug7,
    Unknown,
}

impl ChordQuality {
    /// Get roman numeral representation for this quality
    fn roman_suffix(&self) -> &'static str {
        match self {
            ChordQuality::Major => "",
            ChordQuality::Minor => "",
            ChordQuality::Diminished => "°",
            ChordQuality::Augmented => "+",
            ChordQuality::Dom7 => "7",
            ChordQuality::Maj7 => "maj7",
            ChordQuality::Min7 => "7",
            ChordQuality::HalfDim7 => "ø7",
            ChordQuality::Dim7 => "°7",
            ChordQuality::MinMaj7 => "mM7",
            ChordQuality::AugMaj7 => "+M7",
            ChordQuality::Aug7 => "+7",
            ChordQuality::Unknown => "?",
        }
    }

    /// Is this quality uppercase (major-like) or lowercase (minor-like)?
    fn is_uppercase(&self) -> bool {
        matches!(
            self,
            ChordQuality::Major
                | ChordQuality::Augmented
                | ChordQuality::Dom7
                | ChordQuality::Maj7
                | ChordQuality::AugMaj7
                | ChordQuality::Aug7
        )
    }
}

/// Determine chord quality from intervals relative to root (in semitones)
fn intervals_to_quality(intervals: &[u8]) -> ChordQuality {
    match intervals {
        // Triads
        [0, 4, 7] => ChordQuality::Major,
        [0, 3, 7] => ChordQuality::Minor,
        [0, 3, 6] => ChordQuality::Diminished,
        [0, 4, 8] => ChordQuality::Augmented,
        // 7th chords
        [0, 4, 7, 10] => ChordQuality::Dom7,
        [0, 4, 7, 11] => ChordQuality::Maj7,
        [0, 3, 7, 10] => ChordQuality::Min7,
        [0, 3, 6, 10] => ChordQuality::HalfDim7,
        [0, 3, 6, 9] => ChordQuality::Dim7,
        [0, 3, 7, 11] => ChordQuality::MinMaj7,
        [0, 4, 8, 11] => ChordQuality::AugMaj7,
        [0, 4, 8, 10] => ChordQuality::Aug7,
        _ => ChordQuality::Unknown,
    }
}

/// Harmonic function for progression generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HarmonicFunction {
    Tonic,
    Predominant,
    Dominant,
}

/// A diatonic chord built from a scale
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DiatonicChord {
    degree: u8,           // 1-7
    root_note: Note,
    notes: Vec<Note>,     // Chord tones
    quality: ChordQuality,
    roman: String,        // "I", "ii", "V7", "vii°"
}

impl DiatonicChord {
    /// Get the harmonic function of this chord based on its degree
    fn function(&self) -> HarmonicFunction {
        match self.degree {
            1 | 3 | 6 => HarmonicFunction::Tonic,
            2 | 4 => HarmonicFunction::Predominant,
            5 | 7 => HarmonicFunction::Dominant,
            _ => HarmonicFunction::Tonic,
        }
    }
}

/// Build diatonic chords from a 7-note scale
fn build_diatonic_chords(root: &Note, scale_notes: &[u8], use_sevenths: bool) -> Vec<DiatonicChord> {
    if scale_notes.len() != 7 {
        return vec![];
    }

    let roman_numerals = ["I", "II", "III", "IV", "V", "VI", "VII"];
    let root_midi = root.midi_value();

    let mut chords = Vec::new();

    for degree in 0..7 {
        // Build chord by stacking thirds: root, 3rd, 5th, (7th)
        let indices: Vec<usize> = if use_sevenths {
            vec![degree, (degree + 2) % 7, (degree + 4) % 7, (degree + 6) % 7]
        } else {
            vec![degree, (degree + 2) % 7, (degree + 4) % 7]
        };

        // Get the semitone offsets for each chord tone
        let chord_semitones: Vec<u8> = indices
            .iter()
            .map(|&i| {
                let base = scale_notes[i];
                // Handle octave wrapping
                if i < degree {
                    base + 12
                } else {
                    base
                }
            })
            .collect();

        // Calculate intervals relative to chord root
        let chord_root_offset = scale_notes[degree];
        let intervals: Vec<u8> = chord_semitones
            .iter()
            .map(|&s| (s as i16 - chord_root_offset as i16).rem_euclid(12) as u8)
            .collect();

        // Build actual notes
        let notes: Vec<Note> = chord_semitones
            .iter()
            .filter_map(|&offset| midi_to_note(root_midi.saturating_add(offset)).ok())
            .collect();

        let quality = intervals_to_quality(&intervals);

        // Build roman numeral
        let base_roman = roman_numerals[degree];
        let roman = if quality.is_uppercase() {
            format!("{}{}", base_roman, quality.roman_suffix())
        } else {
            format!("{}{}", base_roman.to_lowercase(), quality.roman_suffix())
        };

        let chord_root = midi_to_note(root_midi.saturating_add(chord_root_offset))
            .unwrap_or(*root);

        chords.push(DiatonicChord {
            degree: (degree + 1) as u8,
            root_note: chord_root,
            notes,
            quality,
            roman,
        });
    }

    chords
}

/// Weighted transition probabilities for Markov chain
struct TransitionWeights {
    tonic: f32,
    predominant: f32,
    dominant: f32,
}

impl TransitionWeights {
    fn from_function(func: HarmonicFunction) -> Self {
        match func {
            HarmonicFunction::Tonic => TransitionWeights {
                tonic: 0.30,
                predominant: 0.40,
                dominant: 0.30,
            },
            HarmonicFunction::Predominant => TransitionWeights {
                tonic: 0.25,
                predominant: 0.15,
                dominant: 0.60,
            },
            HarmonicFunction::Dominant => TransitionWeights {
                tonic: 0.70,
                predominant: 0.15,
                dominant: 0.15,
            },
        }
    }
}

/// Generate a chord progression using weighted Markov transitions
fn generate_progression<'a, R: Rng>(
    chords: &'a [DiatonicChord],
    length: usize,
    rng: &mut R,
) -> Vec<&'a DiatonicChord> {
    if chords.is_empty() || length == 0 {
        return vec![];
    }

    let mut progression = Vec::with_capacity(length);

    // Start on I (tonic)
    let tonic = &chords[0];
    progression.push(tonic);

    // Group chords by function
    let tonic_chords: Vec<&DiatonicChord> = chords
        .iter()
        .filter(|c| c.function() == HarmonicFunction::Tonic)
        .collect();
    let predominant_chords: Vec<&DiatonicChord> = chords
        .iter()
        .filter(|c| c.function() == HarmonicFunction::Predominant)
        .collect();
    let dominant_chords: Vec<&DiatonicChord> = chords
        .iter()
        .filter(|c| c.function() == HarmonicFunction::Dominant)
        .collect();

    // Generate middle chords
    for i in 1..length {
        let current = progression[i - 1];
        let weights = TransitionWeights::from_function(current.function());

        // For second-to-last chord, bias toward dominant for resolution
        let (t_weight, p_weight, _d_weight) = if i == length - 2 {
            (0.15, 0.15, 0.70)
        } else if i == length - 1 {
            // Last chord should be tonic
            (1.0, 0.0, 0.0)
        } else {
            (weights.tonic, weights.predominant, weights.dominant)
        };

        let roll: f32 = rng.gen();
        let next_chord = if roll < t_weight {
            tonic_chords.choose(rng).copied()
        } else if roll < t_weight + p_weight {
            predominant_chords.choose(rng).copied()
        } else {
            dominant_chords.choose(rng).copied()
        };

        if let Some(chord) = next_chord {
            progression.push(chord);
        } else {
            // Fallback to any chord
            if let Some(chord) = chords.choose(rng) {
                progression.push(chord);
            }
        }
    }

    progression
}

/// Humanize velocity by adding random variance
fn humanize_vel<R: Rng>(base_velocity: f32, humanize_amount: f32, rng: &mut R) -> f32 {
    if humanize_amount <= 0.0 {
        return base_velocity;
    }
    // humanize_amount of 1.0 means ±0.5 variance (full range)
    let max_variance = humanize_amount * 0.5;
    let jitter = rng.gen_range(-max_variance..=max_variance);
    (base_velocity + jitter).clamp(0.01, 1.0)
}

/// Add a strummed chord to the sequence
#[allow(clippy::too_many_arguments)]
fn add_strummed_chord<R: Rng>(
    seq: &mut Sequence,
    notes: &[Note],
    time: Time,
    duration: Time,
    velocity: f32,
    humanize_velocity: f32,
    strum_factor: f32,
    time_sig: &TimeSignature,
    rng: &mut R,
) {
    // Convert strum_factor (0-1) to ticks: 1.0 = full beat per note
    let max_strum_ticks = time_sig.beat_time(1.0).ticks;
    let strum_ticks = (max_strum_ticks as f32 * strum_factor) as u32;

    if strum_ticks == 0 {
        // No strum: add all notes as a chord at the same time
        let chord_notes: Vec<(Note, f32, Time)> = notes
            .iter()
            .map(|note| {
                let vel = humanize_vel(velocity, humanize_velocity, rng);
                (*note, vel, duration)
            })
            .collect();
        seq.add_chord(time, chord_notes);
    } else {
        // Strum: add each note with an offset
        for (i, note) in notes.iter().enumerate() {
            // Calculate strum offset with humanized variance (±30%)
            let base_offset = (i as u32) * strum_ticks;
            let variance = (strum_ticks as f32 * 0.3) as i32;
            let jitter = if variance > 0 {
                rng.gen_range(-variance..=variance)
            } else {
                0
            };
            let humanized_offset = (base_offset as i32 + jitter).max(0) as u32;

            let note_time = Time {
                ticks: time.ticks + humanized_offset,
            };

            // Shorten duration to account for strum offset
            let note_duration = if humanized_offset < duration.ticks {
                Time {
                    ticks: duration.ticks - humanized_offset,
                }
            } else {
                Time { ticks: 1 }
            };

            let vel = humanize_vel(velocity, humanize_velocity, rng);
            seq.add_note(note_time, *note, vel, note_duration);
        }
    }
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

fn random_root_note<R: Rng>(rng: &mut R) -> Note {
    let midi: u8 = rng.gen_range(48..=60); // C3 to C4
    midi_to_note(midi).expect("Invalid random MIDI note")
}


fn note_name(note: &Note) -> String {
    // Convert MIDI value to canonical note name like "Eb3", "C4", "A3"
    let midi = note.midi_value();
    let pitch_names = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];
    let pitch = pitch_names[(midi % 12) as usize];
    let octave = (midi / 12).saturating_sub(1);
    format!("{}{}", pitch, octave)
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
    new_smf
        .write_std(&mut output)
        .expect("Failed to write MIDI");
    output
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
                eprintln!("Usage: chord_prog [OPTIONS] [SOUNDFONT]");
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

/// Find a 7-note scale suitable for chord building
fn find_seven_note_scale<R: Rng>(rng: &mut R) -> Option<(String, Vec<u8>)> {
    let mut names = get_scale_names();
    names.sort();
    names.shuffle(rng);

    for name in names {
        if let Ok(notes) = get_scale_notes(&name) {
            if notes.len() == 7 {
                return Some((name, notes));
            }
        }
    }
    None
}

#[derive(Parser)]
#[command(name = "chord_prog")]
#[command(about = "Generate chord progressions from any 7-note scale using harmonic grammar.")]
#[command(version)]
struct Cli {
    /// Scale name (case-insensitive). Random 7-note scale if not specified.
    #[arg(short, long)]
    scale: Option<String>,

    /// Root note as MIDI value (60 = C4) or note name (e.g., "C4", "A3"). Random if not specified.
    #[arg(short = 'r', long)]
    root: Option<String>,

    /// Number of chords in the progression
    #[arg(short, long, default_value = "8")]
    length: usize,

    /// Strum delay factor (0.0 = simultaneous, 1.0 = whole beat per note). Mutually exclusive with --arp.
    #[arg(long, default_value = "0.0")]
    strum: f32,

    /// Arpeggiate chords with this pattern instead of playing simultaneously
    #[arg(short = 'a', long, value_enum)]
    arp: Option<ArpPattern>,

    /// Groove pattern for arpeggio timing (use --list-grooves to see options)
    #[arg(short = 'g', long)]
    groove: Option<String>,

    /// Fill density for extra scale notes (0.0 = none, 1.0 = maximum)
    #[arg(short = 'f', long, default_value = "0.0")]
    fill: f32,

    /// List available groove patterns and exit
    #[arg(long)]
    list_grooves: bool,

    /// Base velocity for MIDI notes (0.0-1.0)
    #[arg(short = 'v', long, default_value = "0.7")]
    velocity: f32,

    /// Velocity humanization amount (0.0 = none, 1.0 = full range variance)
    #[arg(long, default_value = "0.0")]
    humanize_velocity: f32,

    /// Use 7th chords instead of triads
    #[arg(long)]
    sevenths: bool,

    /// Tempo in beats per minute
    #[arg(short = 't', long, default_value = "80")]
    bpm: u32,

    /// Beats per chord
    #[arg(short, long, default_value = "4")]
    beats: u32,

    /// Seed for reproducible random generation
    #[arg(long)]
    seed: Option<u64>,

    /// Save to .mid file instead of playback
    #[arg(short, long)]
    output: Option<String>,

    /// Path to a SoundFont file (searches system directories if not provided)
    soundfont: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let cli = Cli::parse();

    // Handle --list-grooves
    if cli.list_grooves {
        println!("Available grooves:");
        for name in list_grooves() {
            println!("  {}", name);
        }
        return Ok(());
    }

    // Validate groove if specified
    let groove = if let Some(ref name) = cli.groove {
        match get_groove(name) {
            Some(g) => Some(g),
            None => {
                eprintln!("Unknown groove '{}'. Use --list-grooves to see options.", name);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    // Initialize RNG - use separate streams for different purposes
    let seed = cli.seed.unwrap_or_else(rand::random);
    let mut selection_rng = ChaCha8Rng::seed_from_u64(seed);
    let mut progression_rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(1000));
    let mut strum_rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(2000));
    let mut arp_rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(3000));

    // Get or generate scale
    let (scale_name, scale_notes) = if let Some(ref name) = cli.scale {
        let notes = get_scale_notes(name)?;
        if notes.len() != 7 {
            eprintln!(
                "Scale '{}' has {} notes, but chord progressions require exactly 7 notes.",
                name,
                notes.len()
            );
            std::process::exit(1);
        }
        (name.clone(), notes)
    } else {
        match find_seven_note_scale(&mut selection_rng) {
            Some((name, notes)) => (name, notes),
            None => {
                eprintln!("Could not find a 7-note scale");
                std::process::exit(1);
            }
        }
    };

    // Get or generate root note
    let root = if let Some(ref r) = cli.root {
        parse_root_note(r)?
    } else {
        random_root_note(&mut selection_rng)
    };

    // Build diatonic chords
    let chords = build_diatonic_chords(&root, &scale_notes, cli.sevenths);
    if chords.is_empty() {
        eprintln!("Could not build chords from scale");
        std::process::exit(1);
    }

    // Generate progression (uses separate RNG for reproducibility)
    let progression = generate_progression(&chords, cli.length, &mut progression_rng);

    // Display info
    let scale = get_scale(&scale_name)?;
    println!("Scale: {} | Root: {} | Seed: {}", scale.name, note_name(&root), seed);
    println!(
        "Chord types: {} | BPM: {} | Beats per chord: {}",
        if cli.sevenths { "7ths" } else { "triads" },
        cli.bpm,
        cli.beats
    );

    // Display available chords
    print!("Available chords: ");
    for (i, chord) in chords.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{}", chord.roman);
    }
    println!();

    // Display progression
    print!("Progression: ");
    for (i, chord) in progression.iter().enumerate() {
        if i > 0 {
            print!(" - ");
        }
        print!("{}", chord.roman);
    }
    println!();

    // Print reproducible command
    let mut cmd = format!(
        "chord_prog --scale '{}' --root {} --seed {} --length {} --bpm {} --beats {}",
        scale_name.to_lowercase(),
        note_name(&root),
        seed,
        cli.length,
        cli.bpm,
        cli.beats
    );
    if let Some(arp) = cli.arp {
        let arp_name = match arp {
            ArpPattern::Up => "up",
            ArpPattern::Down => "down",
            ArpPattern::UpDown => "up-down",
            ArpPattern::DownUp => "down-up",
            ArpPattern::Random => "random",
            ArpPattern::Converge => "converge",
            ArpPattern::Diverge => "diverge",
            ArpPattern::PingPong => "ping-pong",
            ArpPattern::Thirds => "thirds",
            ArpPattern::Skip => "skip",
            ArpPattern::Walk => "walk",
            ArpPattern::Pairs => "pairs",
            ArpPattern::Triplets => "triplets",
            ArpPattern::Spiral => "spiral",
            ArpPattern::Leaps => "leaps",
            ArpPattern::Zipper => "zipper",
        };
        cmd.push_str(&format!(" --arp {}", arp_name));
    }
    if let Some(ref g) = cli.groove {
        cmd.push_str(&format!(" --groove {}", g));
    }
    if cli.fill > 0.0 {
        cmd.push_str(&format!(" --fill {}", cli.fill));
    }
    if cli.strum > 0.0 && cli.arp.is_none() {
        cmd.push_str(&format!(" --strum {}", cli.strum));
    }
    if (cli.velocity - 0.7).abs() > 0.001 {
        cmd.push_str(&format!(" --velocity {}", cli.velocity));
    }
    if cli.humanize_velocity > 0.0 {
        cmd.push_str(&format!(" --humanize-velocity {}", cli.humanize_velocity));
    }
    if cli.sevenths {
        cmd.push_str(" --sevenths");
    }
    if let Some(ref output) = cli.output {
        cmd.push_str(&format!(" --output '{}'", output));
    }
    if let Some(ref sf) = cli.soundfont {
        cmd.push_str(&format!(" '{}'", sf));
    }
    println!("Repeat: {}", cmd);

    // Build sequence
    let time_sig = common_time();
    let mut seq = Sequence::new(&format!("{} Chord Progression", scale.name), time_sig)?;

    let chord_duration = time_sig.beat_time(cli.beats as f32);
    let velocity = cli.velocity.clamp(0.0, 1.0);
    let humanize_velocity = cli.humanize_velocity.clamp(0.0, 1.0);
    let strum = cli.strum.clamp(0.0, 1.0);
    let fill = cli.fill.clamp(0.0, 1.0);

    // Build scale notes for fill
    let scale_notes_full: Vec<Note> = scale_notes
        .iter()
        .filter_map(|&offset| midi_to_note(root.midi_value().saturating_add(offset)).ok())
        .collect();

    let mut current_time = Time { ticks: 0 };

    if let Some(arp_pattern) = cli.arp {
        // Arpeggio mode
        for chord in &progression {
            let arp_notes = apply_arp_pattern(&mut arp_rng, &chord.notes, arp_pattern);

            if let Some(ref groove) = groove {
                // Apply groove timing
                let mut note_idx = 0;
                let mut groove_time = current_time;
                let end_time = Time { ticks: current_time.ticks + chord_duration.ticks };

                // When looping, skip last note if it equals first (avoids double note at loop point)
                let loop_len = if arp_notes.len() > 1
                    && arp_notes.last() == arp_notes.first()
                {
                    arp_notes.len() - 1
                } else {
                    arp_notes.len()
                };

                while groove_time.ticks < end_time.ticks && !arp_notes.is_empty() {
                    for step in &groove.steps {
                        if groove_time.ticks >= end_time.ticks {
                            break;
                        }

                        let step_duration = time_sig.beat_time(step.beats);
                        let remaining = end_time.ticks - groove_time.ticks;
                        let actual_duration = Time { ticks: step_duration.ticks.min(remaining) };

                        if !step.is_rest {
                            let main_note = arp_notes[note_idx % loop_len];

                            // With high fill, add grace note before main note
                            if fill > 0.3 && arp_rng.gen::<f32>() < fill * 0.6 {
                                if let Some(&grace) = scale_notes_full.choose(&mut arp_rng) {
                                    let grace_dur = Time { ticks: actual_duration.ticks / 6 };
                                    let grace_time = Time {
                                        ticks: groove_time.ticks.saturating_sub(grace_dur.ticks),
                                    };
                                    let vel = humanize_vel(velocity * 0.5, humanize_velocity, &mut arp_rng);
                                    seq.add_note(grace_time, grace, vel, grace_dur * 0.8);
                                }
                            }

                            // Decide whether to replace with fill note
                            let use_fill = fill > 0.0 && arp_rng.gen::<f32>() < fill * 0.5;
                            let note = if use_fill && !scale_notes_full.is_empty() {
                                *scale_notes_full.choose(&mut arp_rng).unwrap()
                            } else {
                                main_note
                            };

                            // Main note
                            let main_dur = if fill > 0.5 && arp_rng.gen::<f32>() < fill * 0.7 {
                                // Shorter note to make room for embellishments
                                actual_duration * 0.5
                            } else {
                                actual_duration * 0.9
                            };
                            let vel = humanize_vel(
                                velocity * step.velocity,
                                humanize_velocity,
                                &mut arp_rng,
                            );
                            seq.add_note(groove_time, note, vel, main_dur);

                            // With high fill, add passing tone after main note
                            if fill > 0.4 && arp_rng.gen::<f32>() < fill * 0.8 {
                                if let Some(&passing) = scale_notes_full.choose(&mut arp_rng) {
                                    let pass_offset = actual_duration.ticks / 2;
                                    let pass_dur = Time { ticks: actual_duration.ticks / 3 };
                                    let pass_time = Time {
                                        ticks: groove_time.ticks + pass_offset,
                                    };
                                    let vel = humanize_vel(velocity * 0.55, humanize_velocity, &mut arp_rng);
                                    seq.add_note(pass_time, passing, vel, pass_dur * 0.8);
                                }
                            }

                            // Extra improvisational flurry at high fill
                            if fill > 0.7 && arp_rng.gen::<f32>() < fill * 0.5 {
                                let num_extra = arp_rng.gen_range(1..=3);
                                for j in 0..num_extra {
                                    if let Some(&extra) = scale_notes_full.choose(&mut arp_rng) {
                                        let offset = actual_duration.ticks * (j + 1) / 4;
                                        let extra_dur = Time { ticks: actual_duration.ticks / 5 };
                                        let extra_time = Time {
                                            ticks: groove_time.ticks + offset,
                                        };
                                        let vel = humanize_vel(velocity * 0.45, humanize_velocity, &mut arp_rng);
                                        seq.add_note(extra_time, extra, vel, extra_dur * 0.7);
                                    }
                                }
                            }

                            if !use_fill {
                                note_idx += 1;
                            }
                        }

                        groove_time = Time { ticks: groove_time.ticks + step_duration.ticks };
                    }
                }
            } else {
                // No groove - play arp notes evenly within chord duration
                let note_duration = Time {
                    ticks: chord_duration.ticks / arp_notes.len().max(1) as u32,
                };
                let mut note_time = current_time;

                for (i, note) in arp_notes.iter().enumerate() {
                    // Maybe insert fill note before this note
                    if fill > 0.0 && i > 0 && arp_rng.gen::<f32>() < fill * 0.2 {
                        if let Some(&fill_note) = scale_notes_full.choose(&mut arp_rng) {
                            let fill_dur = Time { ticks: note_duration.ticks / 2 };
                            let vel = humanize_vel(velocity * 0.6, humanize_velocity, &mut arp_rng);
                            seq.add_note(
                                Time { ticks: note_time.ticks.saturating_sub(fill_dur.ticks) },
                                fill_note,
                                vel,
                                fill_dur * 0.8,
                            );
                        }
                    }

                    let vel = humanize_vel(velocity, humanize_velocity, &mut arp_rng);
                    seq.add_note(note_time, *note, vel, note_duration * 0.9);
                    note_time = Time { ticks: note_time.ticks + note_duration.ticks };
                }
            }

            current_time = Time { ticks: current_time.ticks + chord_duration.ticks };
        }
    } else {
        // Strum/chord mode (original behavior)
        for chord in &progression {
            add_strummed_chord(
                &mut seq,
                &chord.notes,
                current_time,
                chord_duration,
                velocity,
                humanize_velocity,
                strum,
                &time_sig,
                &mut strum_rng,
            );
            current_time = Time {
                ticks: current_time.ticks + chord_duration.ticks,
            };
        }
    }

    // Convert to MIDI
    let smf = seq.to_midi();
    let mut midi_buffer = Vec::new();
    smf.write_std(&mut midi_buffer)?;
    let midi_with_tempo = add_tempo_to_midi_bytes(&midi_buffer, cli.bpm);

    // Output or play
    if let Some(ref output_path) = cli.output {
        let mut file = File::create(output_path)?;
        file.write_all(&midi_with_tempo)?;
        println!("Saved to: {}", output_path);
    } else {
        let soundfont = load_soundfont(cli.soundfont.as_ref())?;
        println!("Using SoundFont: {}", soundfont.path().display());

        let player = Player::new(soundfont)?;
        player.play_midi_bytes(&midi_with_tempo)?;
    }

    Ok(())
}
