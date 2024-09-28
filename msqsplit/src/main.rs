use dbg_hex::dbg_hex;
use std::{fs::File, io::Write, path::PathBuf};

use core::clap::{self, Parser};

#[derive(Parser)]
#[command(version)]
struct Args {
    /// msq file to read
    input: PathBuf,
    /// Whether to display debug information or not
    #[clap(long, short)]
    debug: bool,
}

fn main() {
    core::init();

    let args = Args::parse();

    for file_path in core::get_files(&args.input).unwrap() {
        let bytes = std::fs::read(&file_path).expect("unable to open file");

        let (header, tracks) = convert(bytes, args.debug);
        let folder = file_path.with_extension("");
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
                        header.version.to_ne_bytes().as_slice(),
                    ]
                    .into_iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>(),
                )
                .unwrap();
            output.write_all(&track).unwrap();
        }

        println!("MSQ header for: {file_path:?}");
        println!("Quarter note time: {}", header.quarter_note_time);
        println!("PPQN: {}", header.ppqn);
        println!("BPM: {}", 60_000_000 / header.quarter_note_time);
        println!(
            "Version: {}.{}",
            header.version.to_be_bytes()[0],
            header.version.to_be_bytes()[1]
        );
        println!("Tracks/Channels: {}", header.num_tracks);
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
    version: u16,
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
            version: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            num_tracks: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            _padding: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
        }
    }
}
