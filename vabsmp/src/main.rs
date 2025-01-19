#![allow(dead_code)]

use std::{
    fs::File,
    io::Write,
    num::NonZeroU32,
    ops::Range,
    path::{Path, PathBuf},
    slice::Iter,
};

use core::{
    clap::{self, Parser},
    log::{error, info},
};

#[derive(Parser)]
struct Args {
    vab_path: PathBuf,
    sample_rate: NonZeroU32,
    /// DEFAULT
    #[clap(long)]
    vag: bool,
    #[clap(long)]
    ads: bool,
}

fn main() {
    core::init();

    let args = Args::parse();

    for file in core::get_files(&args.vab_path) {
        convert(&file, args.sample_rate, args.ads);
    }
}

fn convert(path: &Path, sample_rate: NonZeroU32, ads: bool) {
    info!("Reading {path:?}");
    let file = match std::fs::read(path) {
        Ok(f) => f,
        Err(e) => {
            error!("Unable to open file: {e}");
            return;
        }
    };
    let mut file_iter = file.iter();

    let Some(vab_file) = VabFile::parse(&mut file_iter, file.len()) else {
        error!("Unable to parse VAB file");
        return;
    };

    let output_path = path.with_extension("");
    if let Err(e) = std::fs::create_dir(&output_path) {
        error!("Unable to create folder {output_path:?}: {e}");
        return;
    }

    let samples = vab_file.create(&file, sample_rate, ads);
    for (index, sample) in samples.iter().enumerate() {
        let path = output_path.join(format!(
            "{}_{index:04}.{}",
            output_path.file_name().unwrap().to_string_lossy(),
            if ads { "ads" } else { "vag" }
        ));
        let mut out_file = match File::create(&path) {
            Ok(o) => o,
            Err(e) => {
                error!("Unable to create output file {path:?}: {e}");
                continue;
            }
        };
        out_file.write_all(sample).unwrap();
    }
}

#[derive(Debug)]
struct VabFile {
    header: VabHeader,
    programs: Vec<Program>,
    tones: Vec<Vec<Tone>>,
    vag_sizes: Vec<usize>,
    vag_ranges: Vec<Range<usize>>,
}

impl VabFile {
    fn parse(bytes: &mut Iter<u8>, file_len: usize) -> Option<Self> {
        let Some(header) = VabHeader::parse(bytes) else {
            error!("Unable to parse header");
            return None;
        };
        if (file_len as u32) < header.total_size {
            error!("File size mismatch!");
            return None;
        }

        let mut programs = Vec::with_capacity(header.programs_number as usize);
        let mut program_space = 0;
        while programs.len() < header.programs_number as usize {
            if let Some(program) = Program::parse(bytes) {
                programs.push(program);
            }
            program_space += 1;
        }
        for _ in 0..16 * (128 - program_space) {
            bytes.next()?;
        }

        let tones = programs
            .iter()
            .map(|program| {
                let tones = (0..program.tones_number)
                    .map(|_| Tone::parse(bytes))
                    .collect::<Option<Vec<_>>>()?;

                for _ in 0..32 * (16 - program.tones_number as usize) {
                    bytes.next()?;
                }

                Some(tones)
            })
            .collect::<Option<Vec<_>>>()?;

        info!("Samples found: {}", header.vags_number);

        bytes.next()?;
        bytes.next()?;

        let vag_sizes = (0..header.vags_number)
            .map(|_| Some(u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as usize * 8))
            .collect::<Option<Vec<_>>>()?;
        for _ in 0..512 - vag_sizes.len() * 2 - 2 {
            bytes.next()?;
        }

        let start_of_samples = file_len - bytes.as_ref().len();
        let vag_ranges = vag_sizes
            .iter()
            .fold((vec![], start_of_samples), |(mut acc, cursor), size| {
                acc.push(cursor..cursor + *size);
                (acc, cursor + *size)
            })
            .0;

        Some(Self {
            header,
            programs,
            tones,
            vag_sizes,
            vag_ranges,
        })
    }

    fn create(&self, file: &[u8], sample_rate: NonZeroU32, ads: bool) -> Vec<Vec<u8>> {
        self.vag_ranges
            .iter()
            .cloned()
            .map(|range| {
                let mut header = if ads {
                    [
                        [0x53, 0x53, 0x68, 0x64],
                        [0x18, 0, 0, 0],
                        [0x10, 0, 0, 0],
                        sample_rate.get().to_le_bytes(),
                        [1, 0, 0, 0],
                        [0; 4],
                        [0xff; 4],
                        [0xff; 4],
                        [0x53, 0x53, 0x62, 0x64],
                        (range.len() as u32).to_le_bytes(),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                } else {
                    [
                        [0x56, 0x41, 0x47, 0x70],
                        [0, 0, 0, 3],
                        [0; 4],
                        (range.len() as u32).to_be_bytes(),
                        sample_rate.get().to_be_bytes(),
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
                    .collect::<Vec<_>>()
                };
                header.extend_from_slice(&file[range]);
                header
            })
            .collect()
    }
}

#[derive(Debug)]
struct VabHeader {
    magic_number: u32,
    version: u32,
    vab_id: u32,
    total_size: u32,
    _pad0: u16,
    programs_number: u16,
    tones_number: u16,
    vags_number: u16,
    master_volume: u8,
    master_pan: u8,
    bank_attributes_1: u8,
    bank_attributes_2: u8,
    _pad1: u32,
}

impl VabHeader {
    fn parse(bytes: &mut Iter<u8>) -> Option<Self> {
        Some(Self {
            magic_number: u32::from_le_bytes([
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
            ]),
            version: u32::from_le_bytes([
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
            ]),
            vab_id: u32::from_le_bytes([
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
            ]),
            total_size: u32::from_le_bytes([
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
            ]),
            _pad0: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            programs_number: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            tones_number: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            vags_number: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            master_volume: *bytes.next()?,
            master_pan: *bytes.next()?,
            bank_attributes_1: *bytes.next()?,
            bank_attributes_2: *bytes.next()?,
            _pad1: u32::from_le_bytes([
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
            ]),
        })
    }
}

#[derive(Debug)]
struct Program {
    tones_number: u8,
    volume: u8,
    priority: u8,
    mode: u8,
    pan: u8,
    _pad0: u8,
    attribute: u16,
    _pad1: u32,
    _pad2: u32,
}

impl Program {
    fn parse(bytes: &mut Iter<u8>) -> Option<Self> {
        let program = Self {
            tones_number: *bytes.next()?,
            volume: *bytes.next()?,
            priority: *bytes.next()?,
            mode: *bytes.next()?,
            pan: *bytes.next()?,
            _pad0: *bytes.next()?,
            attribute: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            _pad1: u32::from_le_bytes([
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
            ]),
            _pad2: u32::from_le_bytes([
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
                *bytes.next()?,
            ]),
        };

        if program.tones_number == 0 {
            None
        } else {
            Some(program)
        }
    }
}

#[derive(Debug)]
struct Tone {
    priority: u8,
    reverb_mode: u8,
    volume: u8,
    pan: u8,
    unity_key: u8,
    pitch_tune: u8,
    key_low: u8,
    key_high: u8,
    vibrato_width: u8,
    vibrato_time: u8,
    port_width: u8,
    port_hold: u8,
    pitch_bend_minimum: u8,
    pitch_bend_maximum: u8,
    _pad0: u8,
    _pad1: u8,
    adsr1: u16,
    adsr2: u16,
    parent_program: u16,
    sample_number: u16,
    _pad2: u16,
    _pad3: u16,
    _pad4: u16,
    _pad5: u16,
}

impl Tone {
    fn parse(bytes: &mut Iter<u8>) -> Option<Self> {
        Some(Self {
            priority: *bytes.next()?,
            reverb_mode: *bytes.next()?,
            volume: *bytes.next()?,
            pan: *bytes.next()?,
            unity_key: *bytes.next()?,
            pitch_tune: *bytes.next()?,
            key_low: *bytes.next()?,
            key_high: *bytes.next()?,
            vibrato_width: *bytes.next()?,
            vibrato_time: *bytes.next()?,
            port_width: *bytes.next()?,
            port_hold: *bytes.next()?,
            pitch_bend_minimum: *bytes.next()?,
            pitch_bend_maximum: *bytes.next()?,
            _pad0: *bytes.next()?,
            _pad1: *bytes.next()?,
            adsr1: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            adsr2: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            parent_program: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            sample_number: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            _pad2: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            _pad3: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            _pad4: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            _pad5: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
        })
    }
}

#[test]
fn test_file() {
    let vab = include_bytes!("../tests/test.vab");
    let vab_file = VabFile::parse(&mut vab.iter(), vab.len()).unwrap();
    assert_eq!(vab_file.header.total_size, vab.len() as u32);
    assert_eq!(vab_file.tones.len(), 2);
}

#[test]
fn test_conversion() {
    let vab = include_bytes!("../tests/test.vab");
    let vab_file = VabFile::parse(&mut vab.iter(), vab.len()).unwrap();
    let samples = vab_file.create(vab, NonZeroU32::new(22050).unwrap(), false);
    assert_eq!(samples[0].len(), 25264);
    assert_eq!(samples[1].len(), 15968);
}

#[test]
fn cursor() {
    let vab = include_bytes!("../tests/test.vab");
    let mut iter = vab.iter();

    let header = VabHeader::parse(&mut iter).unwrap();

    let mut programs = Vec::with_capacity(header.programs_number as usize);
    let mut program_space = 0;
    while programs.len() < header.programs_number as usize {
        if let Some(program) = Program::parse(&mut iter) {
            programs.push(program);
        }
        program_space += 1;
    }
    for _ in 0..16 * (128 - program_space) {
        iter.next().unwrap();
    }

    let _ = programs
        .iter()
        .map(|program| {
            let tones = (0..program.tones_number)
                .map(|_| Tone::parse(&mut iter))
                .collect::<Option<Vec<_>>>()
                .unwrap();

            for _ in 0..32 * (16 - program.tones_number as usize) {
                iter.next().unwrap();
            }

            Some(tones)
        })
        .collect::<Option<Vec<_>>>()
        .unwrap();

    iter.next().unwrap();
    iter.next().unwrap();

    let vag_sizes = (0..header.vags_number)
        .map(|_| {
            Some(u16::from_le_bytes([*iter.next().unwrap(), *iter.next().unwrap()]) as usize * 8)
        })
        .collect::<Option<Vec<_>>>()
        .unwrap();
    for _ in 0..512 - vag_sizes.len() * 2 - 2 {
        iter.next().unwrap();
    }

    let cursor_index = vab.len() - iter.as_slice().len();
    assert_eq!(cursor_index, 0xe20);
    assert_eq!(vab.len() - cursor_index, 0xa0b0);
}
