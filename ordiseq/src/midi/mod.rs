use klib::core::{named_pitch::HasNamedPitch, note::Note, octave::HasOctave, pitch::HasPitch};

pub trait HasMidiValue {
    /// Calculates the MIDI note value for the current `Note`.
    fn midi_value(&self) -> u8;
}

impl HasMidiValue for Note {
    fn midi_value(&self) -> u8 {
        let pitch_offset = self.named_pitch().pitch() as u8;
        let octave = self.octave() as i8;
        12 * (octave + 1) as u8 + pitch_offset
    }
}
