use std::{
    path::{Path, PathBuf},
    process::exit,
};

use simplelog::TermLogger;

pub use clap;
pub use log;

/// Perform initialisation functions that are common across
/// all unLoKable projects. Primarily initalising debugging.
pub fn init() {
    TermLogger::init(
        if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        },
        simplelog::ConfigBuilder::new()
            .set_thread_level(log::LevelFilter::Off)
            .build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();
}

/// Takes a file path and returns either an iterator of the file path,
/// or if the path is to a directory a list of the files in the path.
///
/// Displays an error and quits if there is an error.
pub fn get_files(path: &Path) -> Vec<PathBuf> {
    if path.is_dir() {
        std::fs::read_dir(path)
            .unwrap_or_else(|e| {
                log::error!("Unable to load paths `{:?}`, aborting: {e}", path);
                exit(1)
            })
            .flatten()
            .map(|f| f.path())
            .collect()
    } else {
        std::iter::once(path.to_owned()).collect()
    }
}
