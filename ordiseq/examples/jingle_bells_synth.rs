use ordiseq::prelude::*;
use ordiseq::synth::{SoundFontSource, PREFERRED_SOUNDFONTS};
use std::env;

/// Jingle Bells - synthesized and played directly to audio device
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    // Get the SoundFont path from command line args or find a default
    let args: Vec<String> = env::args().collect();
    let soundfont = if args.len() >= 2 {
        SoundFontSource::from_path(&args[1])?
    } else {
        match SoundFontSource::default_search() {
            Ok(sf) => sf,
            Err(_) => {
                eprintln!("No soundfont provided and none found in system directories.");
                eprintln!("Usage: {} [soundfont.sf2]", args[0]);
                eprintln!("\nSearched directories:");
                for dir in SoundFontSource::default_search_dirs() {
                    eprintln!("  - {}", dir.display());
                }
                eprintln!("\nPreferred soundfonts: {:?}", PREFERRED_SOUNDFONTS);
                eprintln!("\nExample SoundFonts:");
                eprintln!("  - TimGM6mb.sf2 (small, ~6MB)");
                eprintln!("  - FluidR3_GM.sf2 (full GM, ~140MB)");
                std::process::exit(1);
            }
        }
    };

    println!("Using SoundFont: {}", soundfont.path().display());

    // Build the sequence (same as jingle_bells.rs)
    let time_signature = common_time(); // 4/4 96tpqn
    let mut seq = Sequence::new("Jingle Bells", time_signature)?;

    let note = NoteOrRest::Note;
    let v = 0.7; // Constant velocity for all notes
    let r = 0.5; // Note release 1.==legato, <1.==staccato
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
        (NoteOrRest::Rest, 2, 0., 0.),
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

    seq.load(&verse)?;
    seq = seq.transpose(7)?;

    // Play the sequence using the Player
    let player = Player::new(soundfont)?;
    player.play_sequence(&seq)?;

    Ok(())
}
