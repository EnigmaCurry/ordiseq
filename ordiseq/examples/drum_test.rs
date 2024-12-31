use ordiseq::prelude::*;

/// Drums
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let time_signature = TimeSignature::new("3/4", 96)?;
    let mut seq = DrumSequence::new("Drums test1", time_signature)?;

    seq.add_euclidean_track(
        "Kick Drum",
        C,   // Assuming C4 for the kick drum
        8,   // 16 steps
        4,   // 4 pulses
        0,   // No rotation
        0.8, // Velocity of 80%
    );

    // seq.add_track(
    //     "Snare Drum",
    //     D,   // Assuming D4 for the snare drum
    //     16,  // 16 steps
    //     4,   // 4 pulses
    //     0,   // Rotated by 8 steps
    //     0.7, // Velocity of 70%
    // );

    // seq.add_track(
    //     "Hi-hat", F,   // Assuming F4 for the hi-hat
    //     16,  // 16 steps
    //     8,   // 8 pulses
    //     0,   // Rotated by 2 steps
    //     0.6, // Velocity of 60%
    // );

    seq.to_midi().save(&make_filename(&seq.title(), "mid"))?;

    Ok(())
}
