use std::{fs::File, io::Write, path::PathBuf};

use core::clap::{self, Parser};

#[derive(Parser)]
struct Args {
    /// The `vag` file to read from.
    input: PathBuf,
}

fn main() {
    core::init();

    let args = Args::parse();

    for file_path in core::get_files(&args.input) {
        let mut vag_bytes = std::fs::read(&file_path).unwrap();
        assert_eq!(
            vag_bytes[0..4],
            [0x56, 0x41, 0x47, 0x70],
            "invalid magic number"
        );
        let changed = sanitized(&mut vag_bytes);

        print!("{file_path:?}: ");
        if changed != 0 {
            println!("{changed} bad chunks fixed!");
            let mut output = File::create(format!(
                "{}_clean.{}",
                file_path.with_extension("").to_string_lossy(),
                file_path.extension().unwrap().to_string_lossy(),
            ))
            .unwrap();
            output.write_all(&vag_bytes).unwrap();
        } else {
            println!("No bad chunks were found!");
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
