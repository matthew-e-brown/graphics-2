//! Some general helper-functions just for the testing, exercise, and assignment binaries. For the things I need to do
//! in most of my `main` functions but that don't really belong in Gloog.

use log::LevelFilter;
use simple_logger::SimpleLogger;


/// Initialize logging. By default, uses [info-level][LevelFilter::Info] logging. Change this with `RUST_LOG`.
pub fn init_logger() {
    SimpleLogger::new()
        .with_colors(true)
        .with_level(LevelFilter::Info)
        .env()
        .init()
        .unwrap();
}
