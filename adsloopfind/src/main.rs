use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::exit,
};

use core::{
    clap::{self, Parser},
    log::error,
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
        // if let Some((lb, le)) = find_loops(&path, &std::fs::read(&path).unwrap()) {
        if let Some((lb, le)) = std::fs::read(&path)
            .inspect_err(|e| {
                error!("Unable to read file {path:?}, skipping: {e}");
            })
            .ok()
            .and_then(|contents| find_loops(&path, &contents))
        {
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

fn find_loops(path: &Path, ads_file: &[u8]) -> Option<(u32, u32)> {
    if ads_file[0..4] != MAGIC_NUMBER {
        error!(
            "Invalid magic number, expected {MAGIC_NUMBER:?}, found {:?}, in file {path:?}",
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
