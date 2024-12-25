use env_logger;
pub use log::*;
use std::env;

pub fn setup_log() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    match env_logger::try_init() {
        Ok(_) => {
            debug!("Logger initialized.");
        }
        _ => {
            eprintln!("error setting up logger. Was env_logger already initialized?");
        }
    }
}
