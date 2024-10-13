use std::{fs::File, io::Write, num::NonZeroU32, path::PathBuf};

use core::{
    clap::{self, Parser},
    log::{debug, error, info},
};

#[derive(Parser)]
#[clap(version)]
struct Args {
    input: PathBuf,
    channels: NonZeroU32,
    sample_rate: NonZeroU32,
    interleave: u32,
    format: u32,
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    core::init();

    let args = Args::parse();

    if args.channels.get() == 1 && args.interleave != 0 {
        error!("If there is only 1 channel, then interleave cannot be 0");
        return;
    }
    if args.format != 1 && args.format != 16 {
        error!("Only two formats are supported: 1 (PCM_LE16) and 16 (SONY_4BIT_ADPCM)");
        return;
    }

    let file_paths = core::get_files(&args.input);

    for file_path in file_paths {
        info!("Doing {file_path:?}");

        let mut file = match std::fs::read(&file_path) {
            Ok(file) => file,
            Err(e) => {
                error!("Unable to load file `{file_path:?}`, skipping: {e}");
                continue;
            }
        };
        debug!("File length: {}", file.len());
        add_header(&args, &mut file);

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

fn add_header(args: &Args, file: &mut Vec<u8>) {
    file.splice(
        0..0,
        [
            [0x53, 0x53, 0x68, 0x64],
            [0x18, 0, 0, 0],
            args.format.to_le_bytes(),
            args.sample_rate.get().to_le_bytes(),
            args.channels.get().to_le_bytes(),
            args.interleave.to_le_bytes(),
            [0xff; 4],
            [0xff; 4],
            [0x53, 0x53, 0x62, 0x64],
            (file.len() as u32).to_le_bytes(),
        ]
        .into_iter()
        .flatten(),
    );
}

#[test]
fn adsheader_test() {
    let args = Args {
        input: PathBuf::default(),
        channels: NonZeroU32::new(1).unwrap(),
        sample_rate: NonZeroU32::new(8000).unwrap(),
        interleave: 0,
        format: 16,
        output: None,
    };
    let mut file = include_bytes!("../tests/vag_adpcm.bin").to_vec();
    add_header(&args, &mut file);

    assert_eq!(file[0], 0x53);
    assert_eq!(file[1], 0x53);
    assert_eq!(file[2], 0x68);
    assert_eq!(file[3], 0x64);
    assert_eq!(file.len(), 0x410 + 40);
}
