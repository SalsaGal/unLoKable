use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// The `mul` file to read from.
    input: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mul_file = std::fs::read(&args.input).unwrap();
    let mul_iter = mul_file.iter();

    let mut rate_file = File::create(format!(
        "{}_rate.txt",
        args.input.with_extension("").to_string_lossy()
    ))
    .unwrap();
    writeln!(
        &mut rate_file,
        "{}",
        u32::from_le_bytes([mul_file[0], mul_file[1], mul_file[2], mul_file[3]]),
    )
    .unwrap();
}
