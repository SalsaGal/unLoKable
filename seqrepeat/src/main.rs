use std::{
    fs::File,
    io::Write,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use core::clap::{self, Parser};

#[derive(Parser)]
struct Args {
    /// `seq` files to read from.
    input: PathBuf,
    /// The number of the passes in the final file.
    count: NonZeroUsize,
    /// Whether to read from the tempo marker rather than the entire file.
    #[clap(short)]
    tempo_marker: bool,
    /// Whether to read from the loop markers. This is the default.
    #[clap(short)]
    loop_marker: bool,
}

fn main() {
    core::init();

    let args = Args::parse();

    for file in core::get_files(&args.input).unwrap() {
        repeat_file(&file, &args);
    }
}

fn repeat_file(path: &Path, args: &Args) {
    let file = std::fs::read(path).unwrap();
    let mut bytes = file.iter().copied();

    let (loop_start, loop_end) = args
        .loop_marker
        .then_some(find_loops(&file))
        .unwrap_or_default();

    // Check magic number
    let header = Header::load(&mut bytes);
    assert_eq!(
        header.magic,
        [0x70, 0x51, 0x45, 0x53],
        "Invalid magic number",
    );

    let beginning_index = match args.tempo_marker {
        // 0xff51XXXXXX
        true => {
            file.windows(2)
                .enumerate()
                .find(|(_, w)| *w == [0xff, 0x51])
                .unwrap_or_else(|| {
                    eprintln!("No marker found, defaulting to full file");
                    (10, &[])
                })
                .0
                + 5
        }
        false => 15,
    };
    let beginning = &file[0..beginning_index];
    let to_copy = match (loop_start, loop_end) {
        (Some(start), Some(end)) => &file[start + 6..end + 3],
        (Some(start), None) => &file[start + 6..],
        _ => {
            &file[beginning.len()
                ..file
                    .windows(3)
                    .enumerate()
                    .find(|(_, w)| w.len() == 3 && w == &[0xff, 0x2f, 0x00])
                    .unwrap()
                    .0
                    + 3]
        }
    };

    let mut output = Vec::with_capacity(bytes.len());
    output.write_all(beginning).unwrap();
    if let Some(start) = loop_start {
        output.write_all(&file[beginning_index..start + 6]).unwrap();
    }
    for i in 0..args.count.get() {
        output.write_all(to_copy).unwrap();
        if i < args.count.get() - 1 {
            output.splice(
                output.len() - 3..,
                header.dummy_string(&file, args.loop_marker, loop_start),
            );
        }
    }
    if let Some(end) = loop_end {
        output.write_all(&file[end + 3..]).unwrap();
    }

    let mut out = File::create(path.parent().unwrap().join(format!(
        "{}_x{:02}.seq",
        path.file_stem().unwrap().to_string_lossy(),
        args.count
    )))
    .unwrap();
    out.write_all(&output).unwrap();
}

fn find_loops(file: &[u8]) -> (Option<usize>, Option<usize>) {
    let mut start = None;
    let mut end = None;
    for (index, bytes) in file.windows(3).enumerate() {
        if bytes[0] & 0xf0 == 0xb0 && bytes[1] == 0x63 {
            if bytes[2] == 0x14 {
                start = Some(index);
            } else if bytes[2] == 0x1e {
                end = Some(index);
            }
        }
    }

    (start, end)
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

    fn dummy_string(&self, file: &[u8], loop_marker: bool, loop_start: Option<usize>) -> Vec<u8> {
        if let Some(start) = loop_start {
            vec![file[start], 0x63, 0x1e]
        } else if loop_marker {
            vec![0xb0, 0x63, 0x1e]
        } else {
            vec![0xff, 0x51, self.tempo[0], self.tempo[1], self.tempo[2]]
        }
    }
}
