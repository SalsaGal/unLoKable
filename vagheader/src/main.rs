use std::{fs::File, io::Write, num::NonZeroU32, path::PathBuf};

use core::{
    clap::{self, Parser},
    log::{error, info},
};

#[derive(Parser)]
#[clap(version)]
struct Args {
    input: PathBuf,
    sample_rate: NonZeroU32,
    #[clap(long)]
    long: bool,
    #[clap(long)]
    short: bool,
}

fn main() {
    core::init();

    let args = Args::parse();

    for file_path in core::get_files(&args.input) {
        info!("Handling {file_path:?}");
        let file = match std::fs::read(&file_path) {
            Ok(f) => f,
            Err(e) => {
                error!("Unable to open file: {e}");
                continue;
            }
        };

        let output_path = file_path.with_extension("vag");
        let mut output = match File::create(&output_path) {
            Ok(o) => o,
            Err(e) => {
                error!("Unable to create output {output_path:?}: {e}");
                continue;
            }
        };

        let new_file = add_header(&file, &args);
        output.write_all(&new_file).unwrap();
    }
}

fn add_header(file: &[u8], args: &Args) -> Vec<u8> {
    let mut new_file = vec![];
    new_file.extend(
        [
            [0x56, 0x41, 0x47, 0x70],
            [0x0, 0x0, 0x0, 0x3],
            [0; 4],
            (file.len() as u32).to_be_bytes(),
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
        .flatten(),
    );
    if !args.short {
        new_file.extend_from_slice(&[0; 16]);
    }
    new_file.extend_from_slice(file);
    new_file
}

#[test]
fn short() {
    let file = include_bytes!("../tests/silence.bin");
    let vag = add_header(
        file,
        &Args {
            input: PathBuf::new(),
            sample_rate: NonZeroU32::new(22100).unwrap(),
            long: false,
            short: true,
        },
    );
    assert_eq!(vag.len(), file.len() + 48);
}

#[test]
fn long() {
    let file = include_bytes!("../tests/silence.bin");
    let vag = add_header(
        file,
        &Args {
            input: PathBuf::new(),
            sample_rate: NonZeroU32::new(22100).unwrap(),
            long: true,
            short: false,
        },
    );
    assert_eq!(vag.len(), file.len() + 64);
}
