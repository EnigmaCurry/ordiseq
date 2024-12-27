use klib::core::named_pitch::{HasNamedPitch, NamedPitch};
use klib::core::octave::{HasOctave, Octave};
use klib::core::pitch::{HasPitch, Pitch};

use klib::core::note::Note;

pub trait Transposable {
    /// Transposes the note by the given number of semitones.
    fn transpose(self, semitones: i8) -> Self;
}

impl Transposable for Note {
    fn transpose(self, semitones: i8) -> Self {
        // Get the total semitone offset from the start of the scale.
        let current_index = self.octave() as i32 * 12 + self.named_pitch().pitch() as i32;
        let new_index = current_index + semitones as i32;

        // Calculate the new octave and pitch.
        let new_octave = new_index.div_euclid(12);
        let new_pitch_index = new_index.rem_euclid(12);

        // Convert back to `Octave` and `NamedPitch`.
        let new_octave = Octave::try_from(new_octave as u8).expect("Octave out of range");
        let new_pitch =
            NamedPitch::from(Pitch::try_from(new_pitch_index as u8).expect("Pitch out of range"));

        // Return the new `Note`.
        Note::new(new_pitch, new_octave)
    }
}

/// Sequence type for holding a single note or a rest
#[derive(Clone)]
pub enum NoteOrRest {
    Note(klib::core::note::Note),
    Rest,
}

pub trait IntoNoteOrRest {
    fn into_note_or_rest(self) -> NoteOrRest;
}

impl IntoNoteOrRest for klib::core::note::Note {
    fn into_note_or_rest(self) -> NoteOrRest {
        NoteOrRest::Note(self)
    }
}

impl IntoNoteOrRest for NoteOrRest {
    fn into_note_or_rest(self) -> NoteOrRest {
        self
    }
}
