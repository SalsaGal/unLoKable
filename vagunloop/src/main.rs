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
        info!("Handling {file_path:?}");
        let mut vag_bytes = match std::fs::read(&file_path) {
            Ok(v) => v,
            Err(e) => {
                error!("Unable to read file: {e}");
                continue;
            }
        };

        if vag_bytes[0..4] != MAGIC {
            error!("Invalid magic number");
            continue;
        }

        let mut changed_chunks = 0;
        for chunk in vag_bytes[48..].chunks_mut(16) {
            if remove_loop(chunk) {
                changed_chunks += 1;
            }
        }

        if changed_chunks == 0 {
            info!("No markers found");
        } else {
            let out_path = {
                format!(
                    "{}_unlooped.vag",
                    file_path.with_extension("").to_string_lossy()
                )
            };
            let mut out_file = match File::create(out_path) {
                Ok(o) => o,
                Err(e) => {
                    error!("Unable to create output file: {e}");
                    continue;
                }
            };
            out_file.write_all(&vag_bytes).unwrap();

            info!("Removed {changed_chunks} markers");
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
