use std::{fs::File, io::Write, num::NonZeroUsize, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// `seq` to read from.
    input: PathBuf,
    /// The number of the passes in the final file.
    count: NonZeroUsize,
    /// Whether to read from the tempo marker rather than the entire file.
    #[clap(short)]
    tempo: bool,
    /// `seq` to write to.
    #[clap(short)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let file = std::fs::read(&args.input).expect("unable to load file");
    let mut bytes = file.iter().copied();

    // Check magic number
    let header = Header::load(&mut bytes);
    assert_eq!(
        header.magic,
        [0x70, 0x51, 0x45, 0x53],
        "Invalid magic number",
    );

    let beginning = match args.tempo {
        // 0xff51XXXXXX
        true => {
            &file[0..file
                .windows(2)
                .enumerate()
                .find(|(_, w)| *w == [0xff, 0x51])
                .unwrap_or_else(|| {
                    eprintln!("No marker found, defaulting to full file");
                    (10, &[])
                })
                .0
                + 5]
        }
        false => &file[0..15],
    };
    let to_copy = &file[beginning.len()
        ..file
            .windows(3)
            .enumerate()
            .find(|(_, w)| w.len() == 3 && w == &[0xff, 0x2f, 0x00])
            .unwrap()
            .0
            + 3];

    let mut output = Vec::with_capacity(bytes.len());
    output.write_all(beginning).unwrap();
    for i in 0..args.count.get() {
        output.write_all(to_copy).unwrap();
        if i < args.count.get() - 1 {
            output.splice(output.len() - 3.., header.dummy_string());
        }
    }

    let mut out = File::create(
        args.output
            .unwrap_or(args.input.parent().unwrap().join(format!(
                "{}_x{:02}.seq",
                args.input.file_stem().unwrap().to_string_lossy(),
                args.count
            ))),
    )
    .unwrap();
    out.write_all(&output).unwrap();
}

struct Header {
    magic: [u8; 4],
    _version: u32,
    _ppqn: u16,
    tempo: [u8; 3],
    _time_signature: u16,
}

impl Header {
    fn load(bytes: &mut impl Iterator<Item = u8>) -> Self {
        Self {
            magic: [
                bytes.next().unwrap(),
                bytes.next().unwrap(),
                bytes.next().unwrap(),
                bytes.next().unwrap(),
            ],
            _version: u32::from_ne_bytes([
                bytes.next().unwrap(),
                bytes.next().unwrap(),
                bytes.next().unwrap(),
                bytes.next().unwrap(),
            ]),
            _ppqn: u16::from_ne_bytes([bytes.next().unwrap(), bytes.next().unwrap()]),
            tempo: [
                bytes.next().unwrap(),
                bytes.next().unwrap(),
                bytes.next().unwrap(),
            ],
            _time_signature: u16::from_ne_bytes([bytes.next().unwrap(), bytes.next().unwrap()]),
        }
    }

    fn dummy_string(&self) -> [u8; 5] {
        [0xff, 0x51, self.tempo[0], self.tempo[1], self.tempo[2]]
    }
}
