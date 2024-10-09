use std::{fs::File, io::Write, num::NonZeroU32, path::PathBuf};

use core::{
    clap::{self, Parser},
    log::{error, info},
};

#[derive(Parser)]
#[clap(version)]
struct Args {
    input: PathBuf,
    sample_rate: NonZeroU32,
    #[clap(long)]
    long: bool,
    #[clap(long)]
    short: bool,
}

fn main() {
    core::init();

    let args = Args::parse();

    for file_path in core::get_files(&args.input) {
        info!("Handling {file_path:?}");
        let file = match std::fs::read(&file_path) {
            Ok(f) => f,
            Err(e) => {
                error!("Unable to open file: {e}");
                continue;
            }
        };
        let file_len = file.len();

        let output_path = file_path.with_extension("vag");
        let mut output = match File::create(&output_path) {
            Ok(o) => o,
            Err(e) => {
                error!("Unable to create output {output_path:?}: {e}");
                continue;
            }
        };

        output
            .write_all(
                &[
                    [0x56, 0x41, 0x47, 0x70],
                    [0x0, 0x0, 0x0, 0x3],
                    [0; 4],
                    (file_len as u32).to_be_bytes(),
                    (args.sample_rate.get().to_be_bytes()),
                    [0; 4],
                    [0; 4],
                    [0; 4],
                    [0; 4],
                    [0; 4],
                    [0; 4],
                    [0; 4],
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            )
            .unwrap();
        if !args.short {
            output.write_all(&[0; 16]).unwrap();
        }
        output.write_all(&file).unwrap();
    }
}
