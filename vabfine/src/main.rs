#![allow(dead_code)]

use std::{
    fs::File,
    io::Write,
    ops::Range,
    path::{Path, PathBuf},
    slice::IterMut,
};

use core::clap::{self, Parser};

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
    let mut file = std::fs::read(path).unwrap();
    let file_len = file.len();
    let mut file_iter = file.iter_mut();

    VabFile::parse(&mut file_iter, file_len, psx);

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
    fn parse(bytes: &mut IterMut<u8>, file_len: usize, psx: bool) -> Self {
        let header = VabHeader::parse(bytes);
        assert!(
            (file_len as u32) >= header.total_size,
            "File size mismatch!"
        );

        let mut programs = Vec::with_capacity(header.programs_number as usize);
        let mut program_space = 0;
        while programs.len() < header.programs_number as usize {
            if let Some(program) = Program::parse(bytes) {
                programs.push(program);
            }
            program_space += 1;
        }
        for _ in 0..16 * (128 - program_space) {
            bytes.next().unwrap();
        }

        let tones = programs
            .iter()
            .map(|program| {
                let tones = (0..program.tones_number)
                    .map(|_| Tone::parse(bytes, psx))
                    .collect::<Vec<_>>();

                for _ in 0..32 * (16 - program.tones_number as usize) {
                    bytes.next().unwrap();
                }

                tones
            })
            .collect::<Vec<_>>();

        let pitch_finetunings = tones.iter().map(|tones| tones.iter().len()).sum::<usize>();
        let nonzero_finetunings = tones
            .iter()
            .map(|tones| tones.iter().filter(|t| t.pitch_tune != 0).count())
            .sum::<usize>();

        println!("Tones found: {pitch_finetunings}");
        println!("Changed Non-zero Pitch Finetunings: {nonzero_finetunings}");

        bytes.next().unwrap();
        bytes.next().unwrap();

        let vag_sizes: Vec<usize> = (0..header.vags_number)
            .map(|_| {
                u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]) as usize * 8
            })
            .collect();
        for _ in 0..512 - vag_sizes.len() * 2 - 2 {
            bytes.next().unwrap();
        }

        let start_of_samples = file_len - bytes.as_ref().len();
        let vag_ranges = vag_sizes
            .iter()
            .fold((vec![], start_of_samples), |(mut acc, cursor), size| {
                acc.push(cursor..cursor + *size);
                (acc, cursor + *size)
            })
            .0;

        Self {
            header,
            programs,
            tones,
            vag_sizes,
            vag_ranges,
        }
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
    fn parse(bytes: &mut IterMut<u8>) -> Self {
        Self {
            magic_number: u32::from_le_bytes([
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
            ]),
            version: u32::from_le_bytes([
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
            ]),
            vab_id: u32::from_le_bytes([
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
            ]),
            total_size: u32::from_le_bytes([
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
            ]),
            _pad0: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            programs_number: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            tones_number: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            vags_number: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            master_volume: *bytes.next().unwrap(),
            master_pan: *bytes.next().unwrap(),
            bank_attributes_1: *bytes.next().unwrap(),
            bank_attributes_2: *bytes.next().unwrap(),
            _pad1: u32::from_le_bytes([
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
            ]),
        }
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
            tones_number: *bytes.next().unwrap(),
            volume: *bytes.next().unwrap(),
            priority: *bytes.next().unwrap(),
            mode: *bytes.next().unwrap(),
            pan: *bytes.next().unwrap(),
            _pad0: *bytes.next().unwrap(),
            attribute: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            _pad1: u32::from_le_bytes([
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
            ]),
            _pad2: u32::from_le_bytes([
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
                *bytes.next().unwrap(),
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
    fn parse(bytes: &mut IterMut<u8>, psx: bool) -> Self {
        Self {
            priority: *bytes.next().unwrap(),
            reverb_mode: *bytes.next().unwrap(),
            volume: *bytes.next().unwrap(),
            pan: *bytes.next().unwrap(),
            unity_key: *bytes.next().unwrap(),
            pitch_tune: {
                let byte = bytes.next().unwrap();
                if psx {
                    *byte = ((*byte as u32) * 128 / 100) as u8;
                } else {
                    *byte = ((*byte as u32) * 100 / 128) as u8;
                };
                *byte
            },
            key_low: *bytes.next().unwrap(),
            key_high: *bytes.next().unwrap(),
            vibrato_width: *bytes.next().unwrap(),
            vibrato_time: *bytes.next().unwrap(),
            port_width: *bytes.next().unwrap(),
            port_hold: *bytes.next().unwrap(),
            pitch_bend_minimum: *bytes.next().unwrap(),
            pitch_bend_maximum: *bytes.next().unwrap(),
            _pad0: *bytes.next().unwrap(),
            _pad1: *bytes.next().unwrap(),
            adsr1: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            adsr2: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            parent_program: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            sample_number: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            _pad2: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            _pad3: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            _pad4: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            _pad5: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
        }
    }
}
