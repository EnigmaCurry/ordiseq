use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

const PPQ: u16 = 96;

/// A raw MIDI clip note with beat-relative timing.
#[derive(Debug, Clone, Copy)]
pub struct RawClipNote {
    pub note: u8,
    pub channel: u8,
    pub velocity: f32,
    pub start_beats: f32,
    pub duration_beats: f32,
}

/// Convert raw clip data into a Standard MIDI File.
///
/// `length_beats` sets the total clip length (end-of-track position).
/// Notes are specified with beat-relative timing (quarter-note = 1.0 beat).
/// Uses PPQ=96, matching `common_time()`.
pub fn clip_to_midi(length_beats: f32, notes: &[RawClipNote]) -> Smf<'static> {
    let total_ticks = (length_beats * PPQ as f32).round() as u32;
    let mut events: Vec<(u32, TrackEventKind<'static>)> = Vec::new();

    for n in notes {
        let start_ticks = (n.start_beats * PPQ as f32).round() as u32;
        let duration_ticks = (n.duration_beats * PPQ as f32).round() as u32;
        let vel = ((n.velocity * 127.0).round() as u8).min(127);

        events.push((
            start_ticks,
            TrackEventKind::Midi {
                channel: n.channel.into(),
                message: MidiMessage::NoteOn {
                    key: n.note.into(),
                    vel: vel.into(),
                },
            },
        ));

        events.push((
            start_ticks + duration_ticks,
            TrackEventKind::Midi {
                channel: n.channel.into(),
                message: MidiMessage::NoteOff {
                    key: n.note.into(),
                    vel: 0.into(),
                },
            },
        ));
    }

    events.sort_by_key(|&(ticks, _)| ticks);

    let mut track = Vec::new();

    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 24, 8)),
    });

    let mut last_ticks = 0u32;
    for (ticks, kind) in events {
        let delta = ticks - last_ticks;
        track.push(TrackEvent {
            delta: delta.into(),
            kind,
        });
        last_ticks = ticks;
    }

    let end_delta = total_ticks.saturating_sub(last_ticks);
    track.push(TrackEvent {
        delta: end_delta.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    Smf {
        header: midly::Header {
            format: Format::SingleTrack,
            timing: Timing::Metrical(PPQ.into()),
        },
        tracks: vec![track],
    }
}
