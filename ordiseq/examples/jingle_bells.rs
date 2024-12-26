use ordiseq::prelude::*;

/// Jingle Bells
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let sequence = Sequence::new("Jingle Bells", "4/4", 480)?;

    // Write the MIDI file output:
    sequence
        .to_midi()
        .save(&make_filename(&sequence.title(), "mid"))?;

    Ok(())
}
