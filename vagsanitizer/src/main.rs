use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// The `vag` file to read from.
    input: PathBuf,
    /// The output directory
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mut vag_bytes = std::fs::read(&args.input).unwrap();
    let changed = sanitized(&mut vag_bytes);

    if changed {
        let mut output = File::create(args.output.unwrap_or_else(|| {
            format!(
                "{}_clean.{}",
                args.input.with_extension("").to_string_lossy(),
                args.input.extension().unwrap().to_string_lossy(),
            )
            .into()
        }))
        .unwrap();
        output.write_all(&vag_bytes).unwrap();
    } else {
        eprintln!("No changes made");
    }
}

fn sanitized(bytes: &mut [u8]) -> bool {
    let mut last_valid = 0;
    let mut changed = false;
    for line in bytes.chunks_mut(16).skip(48 / 16) {
        if line[0] == 0xff {
            line[0] = last_valid;
            changed = true;
        } else {
            last_valid = line[0];
        }
    }
    changed
}
