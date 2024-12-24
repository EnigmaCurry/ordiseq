use ordiseq::prelude::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();
    info!("hello");
    warn!("warning");
    Ok(())
}
