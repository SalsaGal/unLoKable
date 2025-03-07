use std::{fs::File, io::Write, path::PathBuf};

use core::{
    clap::{self, Parser},
    log::{error, info},
};

#[derive(Parser)]
#[clap(version)]
struct Args {
    /// The `ads` file to read from.
    input: PathBuf,
}

fn main() {
    core::init();

    let args = Args::parse();

    for file_path in core::get_files(&args.input) {
        info!("Unlooping {file_path:?}");

        let mut ads_bytes = std::fs::read(&file_path).unwrap();

        let Some(changed_chunks) = unloop(&mut ads_bytes) else {
            continue;
        };

        if changed_chunks == 0 {
            info!("No markers found");
        } else {
            let out_path = format!(
                "{}_unlooped.ads",
                file_path.with_extension("").to_string_lossy()
            );
            let mut out_file = File::create(out_path).unwrap();
            out_file.write_all(&ads_bytes).unwrap();

            info!("{changed_chunks} markers removed");
        }
    }
}

/// Returns the number of changed chunks
fn unloop(ads_bytes: &mut [u8]) -> Option<usize> {
    if ads_bytes[0..4] != [0x53, 0x53, 0x68, 0x64] {
        error!("Invalid header magic number, skipping");
        return None;
    }

    if ads_bytes[8..12] != 0x10u32.to_le_bytes() {
        error!("Invalid codec, only support Sony 4-bit ADPCM, skipping");
        return None;
    }

    if ads_bytes[32..36] != [0x53, 0x53, 0x62, 0x64] {
        error!("Invalid body magic number, skipping");
        return None;
    }

    let mut changed_chunks = 0;
    for chunk in ads_bytes[40..].chunks_mut(16) {
        if remove_loop(chunk) {
            changed_chunks += 1;
        }
    }

    Some(changed_chunks)
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

#[test]
fn with_loop() {
    let mut file = include_bytes!("../tests/withloop.ads").to_vec();
    assert_eq!(unloop(&mut file), Some(64));
}

#[test]
fn without_loop() {
    let mut file = include_bytes!("../tests/withoutloop.ads").to_vec();
    assert_eq!(unloop(&mut file), Some(2));
}

#[test]
fn invalid() {
    let mut file = include_bytes!("../tests/pcm.ads").to_vec();
    assert_eq!(unloop(&mut file), None);
}
