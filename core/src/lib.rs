use std::path::{Path, PathBuf};

use simplelog::TermLogger;

pub use clap;

/// Perform initialisation functions that are common across
/// all unLoKable projects. Primarily initalising debugging.
pub fn init() {
    TermLogger::init(
        log::LevelFilter::Trace,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();
}

/// Takes a file path and returns either an iterator of the file path,
/// or if the path is to a directory a list of the files in the path.
pub fn get_files(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    Ok(if path.is_dir() {
        std::fs::read_dir(path)?
            .flatten()
            .map(|f| f.path())
            .collect()
    } else {
        std::iter::once(path.to_owned()).collect()
    })
}
