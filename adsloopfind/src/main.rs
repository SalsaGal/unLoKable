use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

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
    let args = Args::parse();

    let files: &mut dyn Iterator<Item = PathBuf> = if args.ads_input.is_dir() {
        &mut args
            .ads_input
            .read_dir()
            .unwrap()
            .flatten()
            .map(|dir| dir.path())
    } else {
        &mut std::iter::once(args.ads_input.clone())
    };

    let mut file = args.output.map(|path| File::create(path).unwrap());

    for path in files {
        if let Some((lb, le)) = find_loops(&std::fs::read(&path).unwrap()) {
            let text = format!(
                "{lb} {le} {}\r\n",
                path.with_extension("wav")
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
            );
            if let Some(file) = &mut file {
                write!(file, "{}", text).unwrap();
            } else {
                print!("{}", text);
            }
        }
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
