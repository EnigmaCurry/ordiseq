use ordiseq::prelude::*;

/// Jingle Bells
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let time_signature = common_time(); // 4/4 96tpqn
    let mut seq = Sequence::new("Jingle Bells", time_signature)?;

    let (note, rest) = (NoteOrRest::Note, NoteOrRest::Rest);
    let verse = vec![
        //  Note  Beats
        (note(E), 2),
        (note(E), 2),
        (note(E), 4), // "jin-gle bells"
        (note(E), 2),
        (note(E), 2),
        (note(E), 4), // "jin-gle bells"
        (note(E), 2),
        (note(G), 2),
        (note(C), 3),
        (note(D), 1),
        (note(E), 8), // "jin-gle all the way"
        (rest, 8),
        (note(F), 2),
        (note(F), 2),
        (note(F), 3),
        (note(F), 1),
        (note(F), 2),
        (note(E), 2),
        (note(E), 2), // "Oh what fun it is to ride"
        (note(E), 1),
        (note(E), 1),
        (note(E), 2),
        (note(D), 2),
        (note(D), 2),
        (note(E), 2),
        (note(D), 4), // "in a one-horse open sleigh"
        (note(G), 4), // "hey"
    ];

    // The duration of the notes above are legato and without rest.
    // To make them stacatto, shorten the duration of each note, but
    // keep the overall timing:
    let release_scale = 0.5; // Each note is held half as long as before.

    // Add each note or rest of the verse to the sequence:
    seq.load(&verse)?;

    //info!("seq: {seq:#?}");

    // Transpose the notes up seven semi-tones:
    seq = seq.transpose(7)?;

    // Write the MIDI file output:
    seq.to_midi().save(&make_filename(&seq.title(), "mid"))?;

    Ok(())
}
