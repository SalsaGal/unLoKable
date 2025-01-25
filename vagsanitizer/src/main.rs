use std::{fs::File, io::Write, path::PathBuf};

use core::{
    clap::{self, Parser},
    log::{error, info},
};

const MAGIC: [u8; 4] = [0x56, 0x41, 0x47, 0x70];

#[derive(Parser)]
struct Args {
    /// The `vag` file to read from.
    input: PathBuf,
}

fn main() {
    core::init();

    let args = Args::parse();

    for file_path in core::get_files(&args.input) {
        info!("Sanitizing {file_path:?}");
        let mut vag_bytes = match std::fs::read(&file_path) {
            Ok(v) => v,
            Err(e) => {
                error!("Unable to read file: {e}");
                continue;
            }
        };
        if vag_bytes[0..4] == MAGIC {
            error!("File is missing the magic number");
            continue;
        }
        let changed = sanitized(&mut vag_bytes);

        if changed != 0 {
            info!("Fixed {changed} bad chunks");
            let mut output = File::create(format!(
                "{}_clean.{}",
                file_path.with_extension("").to_string_lossy(),
                file_path.extension().unwrap().to_string_lossy(),
            ))
            .unwrap();
            output.write_all(&vag_bytes).unwrap();
        } else {
            info!("No bad chunks were found!");
        }
    }
}

fn sanitized(bytes: &mut [u8]) -> usize {
    let mut last_valid = 0;
    let mut changed = 0;
    for line in bytes.chunks_mut(16).skip(48 / 16) {
        if line[0] == 0xff {
            line[0] = last_valid;
            changed += 1;
        } else {
            last_valid = line[0];
        }
    }
    changed
}

#[test]
fn sanitization() {
    let mut file = include_bytes!("../tests/silence.vag").to_vec();
    let changed = sanitized(&mut file);
    assert_eq!(changed, 6);
    assert_eq!(
        format!("{:X}", md5::compute(file)),
        "6B55C00C906E3D11165F0988981ECA1B"
    );
}
