use std::{fs::File, io::Write, path::PathBuf};

use core::clap::{self, Parser};

#[derive(Parser)]
struct Args {
    /// The `ads` file to read from.
    input: PathBuf,
}

fn main() {
    core::init();

    let args = Args::parse();

    for file_path in core::get_files(&args.input) {
        let mut ads_bytes = std::fs::read(&file_path).unwrap();

        if ads_bytes[0..4] != [0x53, 0x53, 0x68, 0x64] {
            eprintln!("Invalid header magic number");
            continue;
        }

        if ads_bytes[8..12] != 0x10u32.to_le_bytes() {
            eprintln!("Invalid codec, only support Sony 4-bit ADPCM");
            continue;
        }

        if ads_bytes[32..36] != [0x53, 0x53, 0x62, 0x64] {
            eprintln!("Invalid body magic number");
            continue;
        }

        let mut changed_chunks = 0;
        for chunk in ads_bytes[40..].chunks_mut(16) {
            if remove_loop(chunk) {
                changed_chunks += 1;
            }
        }

        if changed_chunks == 0 {
            println!("No markers found");
        } else {
            let out_path = format!(
                "{}_unlooped.ads",
                file_path.with_extension("").to_string_lossy()
            );
            let mut out_file = File::create(out_path).unwrap();
            out_file.write_all(&ads_bytes).unwrap();

            println!("{changed_chunks} markers removed");
        }
    }
}

/// Removes the loop and returns true if any change was made
fn remove_loop(bytes: &mut [u8]) -> bool {
    if bytes[1] != 7 && bytes[1] != 0 {
        bytes[1] = 0;
        true
    } else {
        false
    }
}
