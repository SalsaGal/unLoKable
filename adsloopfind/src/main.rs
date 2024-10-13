use std::{fs::File, io::Write, path::PathBuf, process::exit};

use core::{
    clap::{self, Parser},
    log::{error, info},
};

const MAGIC_NUMBER: [u8; 4] = [0x53, 0x53, 0x68, 0x64];

#[derive(Parser)]
struct Args {
    /// The `ads` file to find loops in
    ads_input: PathBuf,
    /// The file to write the loop locations to, writes to STDOUT otherwise.
    #[clap(short)]
    output: Option<PathBuf>,
}

fn main() {
    core::init();

    let args = Args::parse();

    let files = core::get_files(&args.ads_input);

    let mut file = args.output.map(|path| {
        File::create(&path).unwrap_or_else(|e| {
            error!("Unable to create file {path:?}: {e}");
            exit(1)
        })
    });

    for path in files {
        info!("Finding in {path:?}");

        if let Some((loop_begin, loop_end)) = std::fs::read(&path)
            .inspect_err(|e| {
                error!("Unable to read file {path:?}, skipping: {e}");
            })
            .ok()
            .and_then(|contents| find_loops(&contents))
        {
            let text = format!(
                "{loop_begin} {loop_end} {}",
                path.with_extension("wav")
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
            );
            if let Some(file) = &mut file {
                write!(file, "{}\r\n", text).unwrap();
            } else {
                info!("{text}");
            }
        }
    }
}

fn find_loops(ads_file: &[u8]) -> Option<(u32, u32)> {
    if ads_file[0..4] != MAGIC_NUMBER {
        error!(
            "Invalid magic number, expected {MAGIC_NUMBER:?}, found {:?}",
            &ads_file[0..4]
        );
        return None;
    }

    let body_size = load_bytes(&ads_file[0x24..]);
    let codec = load_bytes(&ads_file[8..]);
    if codec != 0x10 {
        return None;
    }

    let channel_number = load_bytes(&ads_file[0x10..]);
    let step_size = 16;

    let body = &ads_file[0x28..];

    body.chunks(step_size as usize)
        .enumerate()
        .find_map(|(i, x)| {
            if x[1] == 6 {
                Some((
                    i as u32 * 28 / channel_number,
                    body_size / 16 * 28 / channel_number - 1,
                ))
            } else {
                None
            }
        })
}

fn load_bytes(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

#[test]
fn load_bytes_test() {
    assert_eq!(load_bytes(&[0, 0, 0, 0]), 0);
    assert_eq!(load_bytes(&[0, 0, 0, 1]), 0x01000000);
    assert_eq!(load_bytes(&[1, 0, 0, 1]), 0x01000001);
}

#[test]
fn without_loop() {
    let loops = find_loops(include_bytes!("../tests/withoutloop.ads"));
    assert_eq!(loops, None);
}

#[test]
fn with_loop() {
    let loops = find_loops(include_bytes!("../tests/withloop.ads"));
    assert_eq!(loops, Some((896, 1791)));
}

#[test]
fn with_loop_multichannel() {
    let loops = find_loops(include_bytes!("../tests/multichannel.ads"));
    assert_eq!(loops, Some((448, 895)));
}
