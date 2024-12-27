use ordiseq::prelude::*;

/// Jingle Bells
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let time_signature = common_time(); // 4/4 96tpqn
    let mut seq = Sequence::new("Jingle Bells", time_signature)?;

    let note = NoteOrRest::Note;
    let v = 0.7; // Constant velocity for all notes
    let r = 0.5; // Note release 1.==legato, <1.==stacatto
    let verse = vec![
        // Note, Beats, Velocity, Release
        (note(E), 2, v, r),
        (note(E), 2, v, r),
        (note(E), 4, v, r), // "jin-gle bells"
        (note(E), 2, v, r),
        (note(E), 2, v, r),
        (note(E), 4, v, r), // "jin-gle bells"
        (note(E), 2, v, r),
        (note(G), 2, v, r),
        (note(C), 3, v, r),
        (note(D), 1, v, r),
        (note(E), 8, v, r), // "jin-gle all the way"
        (NoteOrRest::Rest, 8, 0., 0.),
        (note(F), 2, v, r),
        (note(F), 2, v, r),
        (note(F), 3, v, r),
        (note(F), 1, v, r),
        (note(F), 2, v, r),
        (note(E), 2, v, r),
        (note(E), 2, v, r), // "Oh what fun it is to ride"
        (note(E), 1, v, r),
        (note(E), 1, v, r),
        (note(E), 2, v, r),
        (note(D), 2, v, r),
        (note(D), 2, v, r),
        (note(E), 2, v, r),
        (note(D), 4, v, r), // "in a one-horse open sleigh"
        (note(G), 4, v, r), // "hey"
    ];

    // Add each note or rest of the verse to the sequence:
    seq.load(&verse)?;

    //info!("seq: {seq:#?}");

    // Transpose the notes up seven semi-tones:
    seq = seq.transpose(7)?;

    // Write the MIDI file output:
    seq.to_midi().save(&make_filename(&seq.title(), "mid"))?;

    Ok(())
}
