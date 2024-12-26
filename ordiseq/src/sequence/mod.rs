//! # Sequence
//!
//! Represent a musical sequence of notes and chords.
//! Includes function to export a sequence to a MIDI file.

use crate::{error::OrdiseqError, midi::HasMidiValue};
use klib::core::note::Note;
use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use std::collections::BTreeMap;

use crate::time::TimeSignature;

/// Represents the time in ticks within a sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub ticks: u32,
}

/// Represents a single note in the sequence.
#[derive(Debug, Clone, PartialEq)]
struct SequenceNote {
    pub note: Note,
    pub velocity: f32, // 0->1
    pub duration: u32, // Duration in ticks
}

/// Represents a chord of notes in the sequence.
///
/// All notes in a chord have the same start time, but each may have a
/// different duration.
#[derive(Debug, Clone, PartialEq)]
struct SequenceChord {
    pub sequence_notes: Vec<SequenceNote>,
}

/// Represents an element of the sequence: a note, a chord
#[derive(Debug, Clone, PartialEq)]
enum SequenceElement {
    Note(SequenceNote),
    Chord(SequenceChord),
}

/// Represents a sequence of elements placed at specific times.
#[derive(Debug, Clone)]
pub struct Sequence {
    title: String,
    time_signature: TimeSignature,
    ppq: u16, // Pulses Per Quarter Note
    elements: BTreeMap<Time, SequenceElement>,
}

impl Sequence {
    /// Creates a new empty sequence with the given time signature and PPQ.
    pub fn new(title: &str, time_signature: &str, ppq: u16) -> Result<Self, OrdiseqError> {
        Ok(Self {
            title: title.to_string(),
            time_signature: TimeSignature::new(time_signature)?,
            ppq,
            elements: BTreeMap::new(),
        })
    }

    /// Adds a note to the sequence at a specific time.
    pub fn add_note(&mut self, time: Time, note: Note, velocity: f32, duration: u32) {
        let sequence_note = SequenceNote {
            note,
            velocity,
            duration,
        };
        self.elements
            .insert(time, SequenceElement::Note(sequence_note));
    }

    /// Adds a chord to the sequence at a specific time.
    pub fn add_chord(
        &mut self,
        time: Time,
        notes: Vec<(Note, f32, u32)>, // Vec of (Note, velocity, duration)
    ) {
        let sequence_notes = notes
            .into_iter()
            .map(|(note, velocity, duration)| SequenceNote {
                note,
                velocity,
                duration,
            })
            .collect();
        let chord = SequenceChord { sequence_notes };
        self.elements.insert(time, SequenceElement::Chord(chord));
    }

    /// Converts the sequence into a MIDI `Smf` (Standard MIDI File).
    pub fn to_midi(&self) -> Smf {
        let mut track = Vec::new();

        // Add a time signature event
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
                self.time_signature.beats_per_bar,
                self.time_signature.beat_unit,
                24,
                8,
            )),
        });

        // Convert sequence elements to MIDI events
        let mut last_time_ticks = 0;

        for (&time, element) in &self.elements {
            let delta = (time.ticks - last_time_ticks).into();
            match element {
                SequenceElement::Note(sequence_note) => {
                    // Note On
                    track.push(TrackEvent {
                        delta,
                        kind: TrackEventKind::Midi {
                            channel: 0.into(),
                            message: MidiMessage::NoteOn {
                                key: sequence_note.note.midi_value().into(),
                                vel: ((sequence_note.velocity * 127.0).round() as u8).into(),
                            },
                        },
                    });
                    // Note Off
                    track.push(TrackEvent {
                        delta: sequence_note.duration.into(),
                        kind: TrackEventKind::Midi {
                            channel: 0.into(),
                            message: MidiMessage::NoteOff {
                                key: sequence_note.note.midi_value().into(),
                                vel: 0.into(),
                            },
                        },
                    });
                }
                SequenceElement::Chord(chord) => {
                    for sequence_note in &chord.sequence_notes {
                        // Note On
                        track.push(TrackEvent {
                            delta,
                            kind: TrackEventKind::Midi {
                                channel: 0.into(),
                                message: MidiMessage::NoteOn {
                                    key: sequence_note.note.midi_value().into(),
                                    vel: ((sequence_note.velocity * 127.0).round() as u8).into(),
                                },
                            },
                        });
                    }
                    for sequence_note in &chord.sequence_notes {
                        // Note Off
                        track.push(TrackEvent {
                            delta: sequence_note.duration.into(),
                            kind: TrackEventKind::Midi {
                                channel: 0.into(),
                                message: MidiMessage::NoteOff {
                                    key: sequence_note.note.midi_value().into(),
                                    vel: 0.into(),
                                },
                            },
                        });
                    }
                }
            }
            last_time_ticks = time.ticks;
        }

        // End of track event
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        Smf {
            header: midly::Header {
                format: Format::SingleTrack,
                timing: Timing::Metrical(self.ppq.into()),
            },
            tracks: vec![track],
        }
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }
}
