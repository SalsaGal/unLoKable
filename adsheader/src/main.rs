use std::{fs::File, io::Write, path::PathBuf, process::exit};

use core::{
    clap::{self, Parser},
    log::{debug, error, info},
};

#[derive(Parser)]
#[clap(version)]
struct Args {
    input: PathBuf,
    channels: u32,
    sample_rate: u32,
    interleave: u32,
    format: u32,
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    core::init();

    let args = Args::parse();

    let file_paths = core::get_files(&args.input).unwrap_or_else(|e| {
        error!("Unable to load paths `{:?}`, aborting: {e}", args.input);
        exit(1);
    });

    for file_path in file_paths {
        info!("Doing {file_path:?}");

        let mut file = match std::fs::read(&file_path) {
            Ok(file) => file,
            Err(e) => {
                error!("Unable to load file `{file_path:?}`, skipping: {e}");
                continue;
            }
        };
        let file_len = file.len();
        debug!("File length: {file_len}");

        file.splice(
            0..0,
            [
                [0x53, 0x53, 0x68, 0x64],
                [0x18, 0, 0, 0],
                args.format.to_le_bytes(),
                args.sample_rate.to_le_bytes(),
                args.channels.to_le_bytes(),
                args.interleave.to_le_bytes(),
                [0xff; 4],
                [0xff; 4],
                [0x53, 0x53, 0x62, 0x64],
                (file_len as u32).to_le_bytes(),
            ]
            .into_iter()
            .flatten(),
        );

        let output_path = file_path.with_extension("ads");
        debug!("Writing to {output_path:?}");
        let mut output = match File::create(&output_path) {
            Ok(file) => file,
            Err(e) => {
                error!("Unable to write output to `{output_path:?}`, skipping: {e}");
                continue;
            }
        };
        output.write_all(&file).unwrap_or_else(|e| {
            error!("Unable to write to file `{output_path:?}`: {e}");
        });
    }
}
