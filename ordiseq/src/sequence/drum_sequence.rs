use crate::midi::HasMidiValue;
use crate::{error::OrdiseqError, time::TimeSignature};
use klib::core::note::{HasNoteId, Note};
use log::info;
use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use std::collections::HashMap;

use super::euclidean_rhythm::generate_euclidean_rhythm;

pub struct DrumSequence {
    title: String,                    // Title of the drum sequence
    tracks: HashMap<u128, DrumTrack>, // HashMap keyed by Note ID (u128)
    time_signature: TimeSignature,
}

pub struct DrumTrack {
    #[allow(dead_code)]
    title: String, // Title of the track
    rhythm: Vec<(bool, f64)>, // Euclidean rhythm as (hit: bool, velocity: f64)
    rotation: usize,          // Rotation of the rhythm
}

impl DrumTrack {
    /// Returns the rhythm with the applied rotation
    pub fn rotated_pattern(&self) -> Vec<(bool, f64)> {
        let mut rotated = self.rhythm.clone();
        rotated.rotate_right(self.rotation);
        rotated
    }
}

impl DrumSequence {
    pub fn new(title: &str, time_signature: TimeSignature) -> Result<Self, OrdiseqError> {
        Ok(DrumSequence {
            title: title.to_string(),
            tracks: HashMap::new(),
            time_signature,
        })
    }

    pub fn add_euclidean_track(
        &mut self,
        title: &str,
        instrument: Note,
        steps: usize,
        pulses: usize,
        rotation: usize,
        velocity: f64,
    ) {
        let rhythm = generate_euclidean_rhythm(steps, pulses, velocity);
        info!("{:?}", rhythm);
        self.tracks.insert(
            instrument.id(), // Use Note::id() as the key
            DrumTrack {
                title: title.to_string(),
                rhythm,
                rotation,
            },
        );
    }

    pub fn to_midi(&self) -> Smf {
        let mut midi_track = vec![];

        // Add metadata (e.g., sequence title)
        midi_track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::TrackName(self.title.as_bytes())),
        });

        // Add a time signature event
        midi_track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
                self.time_signature.beats_per_bar,
                self.time_signature.beat_unit,
                24,
                8,
            )),
        });

        // Add each drum track's events
        for (id, drum_track) in &self.tracks {
            let instrument = Note::from_id(*id).expect("Invalid Note ID"); // Convert ID back to Note
            let step_duration = self.time_signature.ticks_per_quarter_note as u32
                / self.time_signature.beats_per_bar as u32;

            let mut previous_time = 0;

            for (step, &(active, velocity)) in drum_track.rotated_pattern().iter().enumerate() {
                if active {
                    let current_time = step as u32 * step_duration;
                    let delta_time = current_time - previous_time;
                    previous_time = current_time;

                    let midi_value = instrument.midi_value();
                    let velocity = (velocity * 127.0) as u8;

                    // Note On
                    midi_track.push(TrackEvent {
                        delta: delta_time.into(),
                        kind: TrackEventKind::Midi {
                            channel: 9.into(), // Channel 10 for percussion
                            message: MidiMessage::NoteOn {
                                key: midi_value.into(),
                                vel: velocity.into(),
                            },
                        },
                    });

                    // Note Off (short duration)
                    midi_track.push(TrackEvent {
                        delta: (step_duration / 2).into(),
                        kind: TrackEventKind::Midi {
                            channel: 9.into(),
                            message: MidiMessage::NoteOff {
                                key: midi_value.into(),
                                vel: 0.into(),
                            },
                        },
                    });
                }
            }
        }

        // End of Track
        midi_track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        // Return the literal Smf
        Smf {
            header: midly::Header {
                format: Format::SingleTrack,
                timing: Timing::Metrical(
                    (self.time_signature.ticks_per_quarter_note as u16).into(),
                ),
            },
            tracks: vec![midi_track],
        }
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }
    pub fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }
}
