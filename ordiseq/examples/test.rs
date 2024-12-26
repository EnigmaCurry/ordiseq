use ordiseq::prelude::*;

/// Jingle Bells
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let time_signature = common_time();
    let mut seq = Sequence::new("test", time_signature)?;

    let verse = vec![(C, 0.5), (E, 1.0), (G, 2.0), (E, 0.5)];

    // Add each note to the sequence:
    let mut start_time = Time { ticks: 0 };
    for (note, duration) in verse {
        let duration = time_signature.beat_time(duration);
        let end_time = Time {
            ticks: start_time.ticks + duration.ticks,
        };
        seq.add_note(start_time, note, 0.7, duration);
        start_time = end_time; // Update start_time for the next note
    }

    info!("seq: {seq:#?}");

    // Write the MIDI file output:
    seq.to_midi().save(&make_filename(&seq.title(), "mid"))?;

    Ok(())
}
