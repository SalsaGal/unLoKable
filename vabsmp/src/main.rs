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

    let header = VabHeader::parse(&mut file_iter);
    if (file.len() as u32) < header.total_size {
        panic!("File size mismatch!");
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
