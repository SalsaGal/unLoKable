use std::path::PathBuf;

use clap::Parser;

const MAGIC_NUMBER: [u8; 4] = [0x53, 0x53, 0x68, 0x64];

#[derive(Parser)]
struct Args {
    /// The `ads` file to find loops in
    ads_input: PathBuf,
}

fn main() {
    let args = Args::parse();
    let ads_file = std::fs::read(&args.ads_input).unwrap();

    if let Some((lb, le)) = find_loops(&ads_file) {
        print!(
            "{lb} {le} {}\r\n",
            args.ads_input
                .with_extension("wav")
                .file_name()
                .unwrap()
                .to_string_lossy()
        );
    }
}

fn find_loops(ads_file: &[u8]) -> Option<(u32, u32)> {
    if ads_file[0..4] != MAGIC_NUMBER {
        eprintln!(
            "Invalid magic number, expected {MAGIC_NUMBER:?}, found {:?}",
            &ads_file[0..4]
        );
        std::process::exit(1);
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
