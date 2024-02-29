use std::{path::PathBuf, slice::Iter};

use clap::Parser;

#[derive(Parser)]
struct Args {
    input: PathBuf,
}

fn main() {
    let args = Args::parse();

    let file = std::fs::read(args.input).unwrap();
    let mut file_iter = file.iter();

    let vab_file = VabFile::parse(&mut file_iter, &file);
    dbg!(vab_file);

    dbg!(file.len() - file_iter.as_slice().len());
}

#[derive(Debug)]
struct VabFile {
    header: VabHeader,
    programs: Vec<Program>,
    tones: Vec<Vec<Tone>>,
    vag_sizes: Vec<u16>,
}

impl VabFile {
    fn parse(bytes: &mut Iter<u8>, file: &[u8]) -> Self {
        let header = VabHeader::parse(bytes);
        assert!(
            (file.len() as u32) >= header.total_size,
            "File size mismatch!"
        );

        let programs: Vec<Program> = (0..header.programs_number)
            .map(|_| Program::parse(bytes))
            .collect();
        for _ in 0..16 * (128 - header.programs_number) {
            bytes.next().unwrap();
        }

        let tones = programs
            .iter()
            .map(|program| {
                let tones = (0..program.tones_number)
                    .map(|_| Tone::parse(bytes))
                    .collect();

                for _ in 0..32 * (16 - program.tones_number as usize) {
                    bytes.next().unwrap();
                }

                tones
            })
            .collect();

        let vag_sizes: Vec<u16> = (0..header.vags_number)
            .map(|_| u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]))
            .skip(1)
            .collect();
        for _ in 0..512 - vag_sizes.len() * 2 - 2 {
            bytes.next().unwrap();
        }

        Self {
            header,
            programs,
            tones,
            vag_sizes,
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
    fn parse(bytes: &mut Iter<u8>) -> Self {
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
    fn parse(bytes: &mut Iter<u8>) -> Self {
        Self {
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
    fn parse(bytes: &mut Iter<u8>) -> Self {
        Self {
            priority: *bytes.next().unwrap(),
            reverb_mode: *bytes.next().unwrap(),
            volume: *bytes.next().unwrap(),
            pan: *bytes.next().unwrap(),
            unity_key: *bytes.next().unwrap(),
            pitch_tune: *bytes.next().unwrap(),
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
