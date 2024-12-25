use klib::core::named_pitch::NamedPitch;
use ordiseq::prelude::*;

/// Circle of Fifths
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let p = NamedPitch::C;
    info!("{:?}", p);
    Ok(())
}
