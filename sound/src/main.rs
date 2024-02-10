use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Args {
    snd_path: PathBuf,
    smp_path: PathBuf,
    #[clap(short, long)]
    dreamcast: bool,
    #[clap(short)]
    cent_tuning: bool,
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
}
