#![allow(dead_code)]

use std::{
    fs::File,
    io::Write,
    ops::Range,
    path::{Path, PathBuf},
    slice::IterMut,
};

use core::{
    clap::{self, Parser},
    log::{error, info},
};

#[derive(Parser)]
struct Args {
    vab_path: PathBuf,
    /// DEFAULT
    #[clap(long)]
    cents: bool,
    #[clap(long)]
    psx: bool,
}

fn main() {
    core::init();

    let args = Args::parse();

    for file in core::get_files(&args.vab_path) {
        fine_tune(&file, args.psx);
    }
}

fn fine_tune(path: &Path, psx: bool) {
    info!("Handling {path:?}");
    let mut file = match std::fs::read(path) {
        Ok(f) => f,
        Err(e) => {
            error!("Unable to open file {path:?}: {e}");
            return;
        }
    };
    let file_len = file.len();
    let mut file_iter = file.iter_mut();

    if VabFile::parse(&mut file_iter, file_len, psx).is_none() {
        return;
    };

    let path_stem = path.with_extension("");
    let out_path = if psx {
        format!("{}_psx.vab", path_stem.to_string_lossy())
    } else {
        format!("{}_cents.vab", path_stem.to_string_lossy())
    };
    let mut out = File::create(out_path).unwrap();
    out.write_all(&file).unwrap();
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
    fn parse(bytes: &mut IterMut<u8>, file_len: usize, psx: bool) -> Option<Self> {
        let header = VabHeader::parse(bytes)?;
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
                    .map(|_| Tone::parse(bytes, psx))
                    .collect::<Option<Vec<_>>>()?;

                for _ in 0..32 * (16 - program.tones_number as usize) {
                    bytes.next()?;
                }

                Some(tones)
            })
            .collect::<Option<Vec<_>>>()?;

        let pitch_finetunings = tones.iter().map(|tones| tones.iter().len()).sum::<usize>();
        let nonzero_finetunings = tones
            .iter()
            .map(|tones| tones.iter().filter(|t| t.pitch_tune != 0).count())
            .sum::<usize>();

        info!("Tones found: {pitch_finetunings}");
        info!("Changed Non-zero Pitch Finetunings: {nonzero_finetunings}");

        bytes.next()?;
        bytes.next()?;

        let vag_sizes: Vec<usize> = (0..header.vags_number)
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
    fn parse(bytes: &mut IterMut<u8>) -> Option<Self> {
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
    fn parse(bytes: &mut IterMut<u8>) -> Option<Self> {
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
    fn parse(bytes: &mut IterMut<u8>, psx: bool) -> Option<Self> {
        Some(Self {
            priority: *bytes.next()?,
            reverb_mode: *bytes.next()?,
            volume: *bytes.next()?,
            pan: *bytes.next()?,
            unity_key: *bytes.next()?,
            pitch_tune: Self::convert(bytes.next()?, psx),
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

    fn convert(byte: &mut u8, psx: bool) -> u8 {
        if psx {
            *byte = ((*byte as f32) * 128.0 / 100.0).round() as u8;
        } else {
            *byte = ((*byte as f32) * 100.0 / 128.0).round() as u8;
        };
        *byte
    }
}

#[test]
fn test_cents() {
    assert_eq!(Tone::convert(&mut 0, false), 0);
    assert_eq!(Tone::convert(&mut 127, false), 99);
}

#[test]
fn test_psx() {
    assert_eq!(Tone::convert(&mut 0, true), 0);
    assert_eq!(Tone::convert(&mut 99, true), 127);
}

#[test]
fn cents_parsing() {
    let mut file = include_bytes!("../tests/test.vab").to_vec();
    let file_length = file.len();
    let vab = VabFile::parse(&mut file.iter_mut(), file_length, false).unwrap();

    assert_eq!(vab.tones[1][1].pitch_tune, 56);
    assert_eq!(vab.tones[1][2].pitch_tune, 19);
    assert_eq!(vab.tones[1][15].pitch_tune, 27);
    assert_eq!(vab.tones.iter().map(|x| x.len()).sum::<usize>(), 17);
}

#[test]
fn psx_parsing() {
    let mut file = include_bytes!("../tests/test.vab").to_vec();
    let file_length = file.len();
    let vab = VabFile::parse(&mut file.iter_mut(), file_length, true).unwrap();

    assert_eq!(vab.tones[1][1].pitch_tune, 92);
    assert_eq!(vab.tones[1][2].pitch_tune, 31);
    assert_eq!(vab.tones[1][15].pitch_tune, 45);
    assert_eq!(vab.tones.iter().map(|x| x.len()).sum::<usize>(), 17);
}
