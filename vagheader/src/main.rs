use std::{fs::File, io::Write, num::NonZeroU32, path::PathBuf};

use core::clap::{self, Parser};

#[derive(Parser)]
#[clap(version)]
struct Args {
    input: PathBuf,
    sample_rate: NonZeroU32,
    #[clap(long)]
    long: bool,
    #[clap(long)]
    short: bool,
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    core::init();

    let args = Args::parse();

    let file = std::fs::read(&args.input).unwrap();
    let file_len = file.len();

    let mut output = File::create(
        args.output
            .unwrap_or_else(|| args.input.with_extension("vag")),
    )
    .unwrap();

    output
        .write_all(
            &[
                [0x56, 0x41, 0x47, 0x70],
                [0x0, 0x0, 0x0, 0x3],
                [0; 4],
                (file_len as u32).to_be_bytes(),
                (args.sample_rate.get().to_be_bytes()),
                [0; 4],
                [0; 4],
                [0; 4],
                [0; 4],
                [0; 4],
                [0; 4],
                [0; 4],
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        )
        .unwrap();
    if !args.short {
        output.write_all(&[0; 16]).unwrap();
    }
    output.write_all(&file).unwrap();
}
