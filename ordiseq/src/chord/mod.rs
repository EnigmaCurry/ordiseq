use crate::midi::HasMidiValue;
use klib::core::{
    base::HasName,
    chord::{Chord, Chordable, HasChord},
    named_pitch::NamedPitch,
    note::Note,
    octave::Octave,
};

const PITCH_CLASSES: [NamedPitch; 12] = [
    NamedPitch::C,
    NamedPitch::CSharp,
    NamedPitch::D,
    NamedPitch::DSharp,
    NamedPitch::E,
    NamedPitch::F,
    NamedPitch::FSharp,
    NamedPitch::G,
    NamedPitch::GSharp,
    NamedPitch::A,
    NamedPitch::ASharp,
    NamedPitch::B,
];

const OCTAVES: [Octave; 11] = [
    Octave::Zero,
    Octave::One,
    Octave::Two,
    Octave::Three,
    Octave::Four,
    Octave::Five,
    Octave::Six,
    Octave::Seven,
    Octave::Eight,
    Octave::Nine,
    Octave::Ten,
];

/// All supported chord type labels, in display order.
pub const CHORD_TYPES: &[&str] = &[
    "Major", "Minor", "Dim", "Aug",
    "Maj7", "7", "m7", "mMaj7",
    "dim7", "m7b5", "Aug7", "AugMaj7",
    "9", "Maj9", "m9", "add9",
    "sus2", "sus4", "6", "m6",
];

/// Convert a MIDI note number (0-127) to a klib `Note`.
pub fn note_from_midi(midi: u8) -> Note {
    let pitch = PITCH_CLASSES[(midi % 12) as usize];
    let octave_index = (midi / 12).saturating_sub(1) as usize;
    let octave = OCTAVES[octave_index.min(OCTAVES.len() - 1)];
    Note::new(pitch, octave)
}

/// Build a chord from a root MIDI note and a chord type label.
///
/// Returns the MIDI note numbers of the chord tones, or an empty vec
/// if the chord type is unrecognized.
pub fn get_chord_notes(root_midi: u8, chord_type: &str) -> Vec<u8> {
    let root = note_from_midi(root_midi);
    let base = Chord::new(root);

    let chord = match chord_type {
        "Major" => base,
        "Minor" => base.minor(),
        "Dim" => base.diminished(),
        "Aug" => base.augmented(),
        "Maj7" => base.maj7(),
        "7" => base.seven(),
        "m7" => base.minor().seven(),
        "mMaj7" => base.minor().maj7(),
        "dim7" => base.diminished().seven(),
        "m7b5" => base.half_diminished(),
        "Aug7" => base.augmented().seven(),
        "AugMaj7" => base.augmented().maj7(),
        "9" => base.nine(),
        "Maj9" => base.maj7().add9(),
        "m9" => base.minor().nine(),
        "add9" => base.add9(),
        "sus2" => base.sus2(),
        "sus4" => base.sus4(),
        "6" => base.add6(),
        "m6" => base.minor().add6(),
        _ => return Vec::new(),
    };

    chord.chord().iter().map(|n| n.midi_value()).collect()
}

/// Detect chords from a set of MIDI note numbers.
///
/// Returns all matching chord names sorted by simplicity (e.g. ["C", "Am/C", ...]),
/// or an empty vec if fewer than 3 notes are provided or no chord is recognized.
pub fn detect_chords(midi_notes: &[u8]) -> Vec<String> {
    if midi_notes.len() < 3 {
        return Vec::new();
    }
    let notes: Vec<Note> = midi_notes.iter().map(|&m| note_from_midi(m)).collect();
    let Ok(chords) = Chord::try_from_notes(&notes) else {
        return Vec::new();
    };
    chords.iter().map(|c| c.name()).collect()
}
