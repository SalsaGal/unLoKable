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
