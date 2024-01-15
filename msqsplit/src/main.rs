use dbg_hex::dbg_hex;
use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Args {
    /// msq file to read
    input: PathBuf,
    /// Whether to display debug information or not
    #[clap(long, short)]
    debug: bool,
    /// Output path of the cds file, defaults to the input with a different extension
    #[clap(long, short)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let bytes = std::fs::read(&args.input).expect("unable to open file");

    let (header, tracks) = convert(bytes, args.debug);
    let folder = args.input.with_extension("");
    std::fs::create_dir(&folder).expect("unable to make output folder");
    for (index, track) in tracks.into_iter().enumerate() {
        let mut output = File::create(folder.join(format!(
            "{}_{index:04}.cds",
            folder.file_name().unwrap().to_string_lossy()
        )))
        .unwrap();
        output
            .write_all(
                &[
                    &[0x51, 0x45, 0x53, 0x61],
                    header.quarter_note_time.to_ne_bytes().as_slice(),
                    header.ppqn.to_ne_bytes().as_slice(),
                    &[0x43, 0x00],
                ]
                .into_iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
            )
            .unwrap();
        output.write_all(&track).unwrap();
    }
}

fn convert(bytes: Vec<u8>, debug: bool) -> (MsqHeader, Vec<Vec<u8>>) {
    let mut bytes_iter = bytes.iter();

    let header = MsqHeader::parse(&mut bytes_iter);
    assert!(
        header.magic == 0x614d_5351 || header.magic == 0x6153_4551,
        "found invalid magic number {:#x}",
        header.magic
    );
    if debug {
        dbg_hex!(&header);
    }

    let track_offsets = bytes_iter
        .take(header.num_tracks as usize * 4)
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|x| u32::from_le_bytes([*x[0], *x[1], *x[2], *x[3]]))
        .collect::<Vec<_>>();

    if debug {
        dbg!(&track_offsets);
    }

    let mut tracks = Vec::with_capacity(track_offsets.len());
    for (i, offset) in track_offsets.iter().copied().enumerate() {
        tracks.push(
            offset as usize
                ..track_offsets
                    .get(i + 1)
                    .copied()
                    .unwrap_or(bytes.len() as u32) as usize,
        );
    }

    if debug {
        dbg!(&tracks);
    }

    (
        header,
        tracks
            .into_iter()
            .map(|x| bytes[x].to_vec())
            .collect::<Vec<_>>(),
    )
}

#[derive(Debug)]
struct MsqHeader {
    magic: u32,
    quarter_note_time: u32,
    ppqn: u16,
    _version: u16,
    num_tracks: u16,
    _padding: u16,
}

impl MsqHeader {
    fn parse<'a>(bytes: &mut impl Iterator<Item = &'a u8>) -> Self {
        MsqHeader {
            magic: u32::from_le_bytes([
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
            ]),
            quarter_note_time: u32::from_le_bytes([
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
            ]),
            ppqn: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            _version: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            num_tracks: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            _padding: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
        }
    }
}
