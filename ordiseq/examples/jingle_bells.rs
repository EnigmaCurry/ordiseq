use ordiseq::prelude::*;

/// Jingle Bells
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let time_signature = common_time();
    let mut seq = Sequence::new("Jingle Bells", time_signature)?;

    let verse = vec![
        (E, 1.0),
        (E, 1.0),
        (E, 2.0), // "jin-gle bells"
        (E, 1.0),
        (E, 1.0),
        (E, 2.0), // "jin-gle bells"
        (E, 1.0),
        (G, 1.0),
        (C, 1.5),
        (D, 0.5),
        (E, 2.0), // "jin-gle all the way"
        (F, 1.0),
        (F, 1.0),
        (F, 1.5),
        (F, 0.5),
        (F, 1.0),
        (E, 1.0),
        (E, 1.0),
        (E, 0.5), // "Oh what fun it is to"
        (E, 0.5),
        (E, 1.0),
        (D, 1.0),
        (D, 1.0),
        (E, 1.0),
        (D, 2.0),
        (G, 2.0), // "ride in a one-horse open sleigh"
    ];

    // Add each note to the sequence:
    let mut start_time = Time { ticks: 0 };
    for (note, duration) in verse {
        let duration = Time {
            ticks: (duration * time_signature.ticks_per_quarter_note as f32) as u32,
        };
        let end_time = Time {
            ticks: start_time.ticks + duration.ticks,
        };
        seq.add_note(start_time, note, 0.7, duration);
        start_time = end_time; // Update start_time for the next note
    }

    // Write the MIDI file output:
    seq.to_midi().save(&make_filename(&seq.title(), "mid"))?;

    Ok(())
}
