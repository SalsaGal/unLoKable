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

    assert_eq!(
        &vag_bytes[0..4],
        &[0x56, 0x41, 0x47, 0x70],
        "Invalid magic number"
    );

    let mut changed_chunks = 0;
    for chunk in vag_bytes[48..].chunks_mut(16) {
        if remove_loop(chunk) {
            changed_chunks += 1;
        }
    }

    if changed_chunks == 0 {
        println!("No markers found");
    } else {
        let out_path = args.output.unwrap_or_else(|| {
            format!(
                "{}_unlooped.vag",
                args.input.with_extension("").to_string_lossy()
            )
            .into()
        });
        let mut out_file = File::create(out_path).unwrap();
        out_file.write_all(&vag_bytes).unwrap();

        println!("{changed_chunks} markers removed");
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
