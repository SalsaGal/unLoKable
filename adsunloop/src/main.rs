use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// The `ads` file to read from.
    input: PathBuf,
    /// The output directory
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mut ads_bytes = std::fs::read(&args.input).unwrap();

    assert_eq!(
        &ads_bytes[0..4],
        &[0x53, 0x53, 0x68, 0x64],
        "Invalid header magic number"
    );

    assert_eq!(
        ads_bytes[8..12],
        0x10u32.to_le_bytes(),
        "Invalid codec, only support Sony 4-bit ADPCM"
    );

    assert_eq!(
        &ads_bytes[32..36],
        &[0x53, 0x53, 0x62, 0x64],
        "Invalid body magic number"
    );

    let mut changed_chunks = 0;
    for chunk in ads_bytes[40..].chunks_mut(16) {
        if remove_loop(chunk) {
            changed_chunks += 1;
        }
    }

    if changed_chunks == 0 {
        println!("No markers found");
    } else {
        let out_path = args.output.unwrap_or_else(|| {
            format!(
                "{}_unlooped.ads",
                args.input.with_extension("").to_string_lossy()
            )
            .into()
        });
        let mut out_file = File::create(out_path).unwrap();
        out_file.write_all(&ads_bytes).unwrap();

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
