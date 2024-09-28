use std::{fs::File, io::Write, path::PathBuf};

use core::clap::{self, Parser};

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
    core::init();

    let args = Args::parse();

    let file_paths: &mut dyn Iterator<Item = PathBuf> = if args.input.is_dir() {
        &mut args
            .input
            .read_dir()
            .unwrap()
            .flatten()
            .map(|dir| dir.path())
    } else {
        &mut std::iter::once(args.input.clone())
    };

    for file_path in file_paths {
        let mut file = std::fs::read(&file_path).unwrap();
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

        let mut output = File::create(file_path.with_extension("ads")).unwrap();
        output.write_all(&file).unwrap();
    }
}
