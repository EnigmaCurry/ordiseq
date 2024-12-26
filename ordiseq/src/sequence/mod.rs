//! # Sequence
//!
//! Represent a musical sequence of notes and chords.
//! Includes function to export a sequence to a MIDI file.

use crate::klib_trait::Transposable;
use crate::time::{Time, TimeSignature};
use crate::{error::OrdiseqError, midi::HasMidiValue, time::calculate_tpqn};
use klib::core::note::Note;
use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use std::collections::BTreeMap;

/// Represents a single note in the sequence.
#[derive(Debug, Clone, PartialEq)]
struct SequenceNote {
    pub note: Note,
    pub velocity: f32, // 0->1
    pub duration: Time,
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
    elements: BTreeMap<Time, SequenceElement>,
}

impl Sequence {
    /// Creates a new empty sequence with the given time signature and PPQ.
    pub fn new(title: &str, time_signature: TimeSignature) -> Result<Self, OrdiseqError> {
        Ok(Self {
            title: title.to_string(),
            time_signature,
            elements: BTreeMap::new(),
        })
    }

    /// Adds a note to the sequence at a specific time.
    pub fn add_note(&mut self, time: Time, note: Note, velocity: f32, duration: Time) {
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
        notes: Vec<(Note, f32, Time)>, // Vec of (Note, velocity, duration)
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

    pub fn transpose(mut self, semitones: i8) -> Result<Self, OrdiseqError> {
        for (_time, element) in &mut self.elements {
            match element {
                SequenceElement::Note(note) => {
                    note.note = note.note.transpose(semitones);
                }
                SequenceElement::Chord(_) => {
                    return Err(OrdiseqError::ChordTranspositionUnsupported);
                }
            }
        }
        Ok(self)
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

        // Collect all note-on and note-off events
        let mut events = Vec::new();

        for (&time, element) in &self.elements {
            match element {
                SequenceElement::Note(sequence_note) => {
                    // Note On
                    events.push((
                        time.ticks,
                        TrackEventKind::Midi {
                            channel: 0.into(),
                            message: MidiMessage::NoteOn {
                                key: sequence_note.note.midi_value().into(),
                                vel: ((sequence_note.velocity * 127.0).round() as u8).into(),
                            },
                        },
                    ));

                    // Note Off
                    let end_time_ticks = time.ticks + sequence_note.duration.ticks;
                    events.push((
                        end_time_ticks,
                        TrackEventKind::Midi {
                            channel: 0.into(),
                            message: MidiMessage::NoteOff {
                                key: sequence_note.note.midi_value().into(),
                                vel: 0.into(),
                            },
                        },
                    ));
                }
                SequenceElement::Chord(chord) => {
                    for sequence_note in &chord.sequence_notes {
                        // Note On
                        events.push((
                            time.ticks,
                            TrackEventKind::Midi {
                                channel: 0.into(),
                                message: MidiMessage::NoteOn {
                                    key: sequence_note.note.midi_value().into(),
                                    vel: ((sequence_note.velocity * 127.0).round() as u8).into(),
                                },
                            },
                        ));
                    }

                    // Note Offs
                    for sequence_note in &chord.sequence_notes {
                        let end_time_ticks = time.ticks + sequence_note.duration.ticks;
                        events.push((
                            end_time_ticks,
                            TrackEventKind::Midi {
                                channel: 0.into(),
                                message: MidiMessage::NoteOff {
                                    key: sequence_note.note.midi_value().into(),
                                    vel: 0.into(),
                                },
                            },
                        ));
                    }
                }
            }
        }

        // Sort events by time ticks
        events.sort_by_key(|&(ticks, _)| ticks);

        // Add sorted events to the track
        let mut last_time_ticks = 0;

        for (time, kind) in events {
            if time < last_time_ticks {
                panic!(
                    "Time ticks are not in increasing order: {} < {}",
                    time, last_time_ticks
                );
            }

            let delta = (time - last_time_ticks).into();
            track.push(TrackEvent { delta, kind });
            last_time_ticks = time;
        }

        // End of track event
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        Smf {
            header: midly::Header {
                format: Format::SingleTrack,
                timing: Timing::Metrical(self.ppq().into()),
            },
            tracks: vec![track],
        }
    }

    fn ppq(&self) -> u16 {
        calculate_tpqn(self.time_signature).unwrap_or(96)
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }
    pub fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }
}
