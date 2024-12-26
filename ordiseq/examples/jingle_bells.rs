use ordiseq::prelude::*;

/// Jingle Bells
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let time_signature = common_time();
    let mut seq = Sequence::new("Jingle Bells", time_signature)?;

    let verse = vec![
        (E, 2.0),
        (E, 2.0),
        (E, 4.0), // "jin-gle bells"
        (E, 2.0),
        (E, 2.0),
        (E, 4.0), // "jin-gle bells"
        (E, 2.0),
        (G, 2.0),
        (C, 3.0),
        (D, 1.0),
        (E, 8.0), // "jin-gle all the way"
        (F, 2.0),
        (F, 2.0),
        (F, 3.0),
        (F, 1.0),
        (F, 2.0),
        (E, 2.0),
        (E, 2.0), // "Oh what fun it is to ride"
        (E, 1.0),
        (E, 1.0),
        (E, 2.0),
        (D, 2.0),
        (D, 2.0),
        (E, 2.0),
        (D, 4.0), // "in a one-horse open sleigh"
        (G, 4.0), // "hey"
    ];

    // The sequence above codes notes as legato with no rests.
    // We should scale each note duration without affecting the timing:
    let release_scale = 0.5; // Each note is half as long as it would be.

    // Add each note to the sequence:
    let mut start_time = Time { ticks: 0 };
    for (note, duration) in verse {
        let duration = time_signature.beat_time(duration);
        let end_time = Time {
            ticks: start_time.ticks + duration.ticks,
        };
        // Add the note while scaling the duration acording to release_scale:
        seq.add_note(start_time, note, 0.7, duration * 0.5);
        start_time = end_time;
    }

    //info!("seq: {seq:#?}");

    // Write the MIDI file output:
    seq.to_midi().save(&make_filename(&seq.title(), "mid"))?;

    Ok(())
}
