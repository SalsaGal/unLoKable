use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
#[clap(version)]
struct Args {
    input: PathBuf,
    channels: u32,
    sample_rate: u32,
    interleave: u32,
    format: u32,
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mut file = std::fs::read(&args.input).unwrap();
    let file_len = file.len();

    file.splice(
        0..0,
        [
            [0x53, 0x53, 0x68, 0x64],
            [0x18, 0, 0, 0],
            args.format.to_le_bytes(),
            args.sample_rate.to_le_bytes(),
            args.channels.to_le_bytes(),
            args.interleave.to_le_bytes(),
            [0xff; 4],
            [0xff; 4],
            [0x53, 0x53, 0x62, 0x64],
            (file_len as u32).to_le_bytes(),
        ]
        .into_iter()
        .flatten(),
    );

    let mut output = File::create(args.input.with_extension("ads")).unwrap();
    output.write_all(&file).unwrap();
}
