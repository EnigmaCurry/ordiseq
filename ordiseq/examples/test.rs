use ordiseq::prelude::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();
    info!("hello");

    let c_chord = Chord::parse("C")?;
    info!("{:?}", c_chord.);

    Ok(())
}
