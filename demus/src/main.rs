use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Platform {
    Dreamcast,
    PC,
}

#[derive(Parser)]
#[command(version)]
struct Args {
    /// msq file to read
    input: PathBuf,
    /// Whether to display debug information or not
    #[clap(short)]
    debug: bool,
    /// What platform to use the format of
    #[clap(short)]
    platform: Platform,
    /// Output path of the cds file, defaults to the input with a different extension
    #[clap(long, short)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
}
