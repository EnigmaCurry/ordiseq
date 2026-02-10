use klib::core::{base::HasName, chord::Chord, named_pitch::NamedPitch, note::Note, octave::Octave};

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

/// Convert a MIDI note number (0-127) to a klib `Note`.
pub fn note_from_midi(midi: u8) -> Note {
    let pitch = PITCH_CLASSES[(midi % 12) as usize];
    let octave_index = (midi / 12).saturating_sub(1) as usize;
    let octave = OCTAVES[octave_index.min(OCTAVES.len() - 1)];
    Note::new(pitch, octave)
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
