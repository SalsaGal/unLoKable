use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Platform {
    Dreamcast,
    PC,
}

#[derive(Parser)]
#[command(version)]
struct Args {
    /// `mus` file to read
    mus_path: PathBuf,
    /// `sam` file to read
    sam_path: PathBuf,
    /// Whether to display debug information or not
    #[clap(short)]
    debug: bool,
    /// What platform to use the format of
    #[clap(short)]
    platform: Platform,
    /// Output path of the cds file, defaults to the input with a different extension
    #[clap(long, short)]
    output: Option<PathBuf>,
}

macro_rules! le_bytes {
    ($bytes: ident) => {
        u32::from_le_bytes([
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
        ])
    };
}

macro_rules! be_bytes {
    ($bytes: ident) => {
        u32::from_be_bytes([
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
        ])
    };
}

macro_rules! float_be_bytes {
    ($bytes: ident) => {
        f32::from_be_bytes([
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
        ])
    };
}

macro_rules! name_bytes {
    ($bytes: ident) => {{
        let mut bytes = (&mut $bytes)
            .take(20)
            .take_while(WaveEntry::valid_char)
            .collect::<Vec<_>>();
        while bytes.len() < 20 {
            bytes.push(0);
        }
        bytes
            .into_iter()
            .map(char::from)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }};
}

fn main() {
    let args = Args::parse();

    let mus_file = std::fs::read(args.mus_path).unwrap();
    let mut mus_bytes = mus_file.into_iter();
    let sam_file = std::fs::read(args.sam_path).unwrap();
    let mut sam_bytes = sam_file.into_iter();

    let header = MusHeader {
        magic: le_bytes!(mus_bytes),
        header_size: be_bytes!(mus_bytes),
        version_number: be_bytes!(mus_bytes),
        reverb_volume: be_bytes!(mus_bytes),
        reverb_type: be_bytes!(mus_bytes),
        reverb_multiply: be_bytes!(mus_bytes),
        num_sequences: be_bytes!(mus_bytes),
        num_labels: be_bytes!(mus_bytes),
        offset_to_labels_offsets_table: be_bytes!(mus_bytes),
        num_waves: be_bytes!(mus_bytes),
        num_programs: be_bytes!(mus_bytes),
        num_presets: be_bytes!(mus_bytes),
    };
    assert_eq!(header.magic, 0x4D757321, "Invalid magic number");
    if args.debug {
        dbg!(&header);
    }

    let msq_tables = (0..header.num_sequences)
        .map(|_| MsqTable {
            index: be_bytes!(mus_bytes),
            offset: be_bytes!(mus_bytes),
        })
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&msq_tables);
    }

    let layers = (0..header.num_presets + header.num_programs)
        .map(|_| be_bytes!(mus_bytes))
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&layers);
    }

    let wave_entries = (0..header.num_waves)
        .map(|_| WaveEntry {
            name: name_bytes!(mus_bytes),
            offset: be_bytes!(mus_bytes),
            loop_begin: be_bytes!(mus_bytes),
            size: be_bytes!(mus_bytes),
            loop_end: be_bytes!(mus_bytes),
            sample_rate: be_bytes!(mus_bytes),
            original_pitch: be_bytes!(mus_bytes),
            loop_info: be_bytes!(mus_bytes),
            snd_handle: be_bytes!(mus_bytes),
        })
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&wave_entries);
    }

    let program_entries = (0..header.num_programs)
        .map(|_| ProgramEntry {
            name: name_bytes!(mus_bytes),
            num_zones: be_bytes!(mus_bytes),
        })
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&program_entries);
    }

    let program_zones = (0..header.num_programs)
        .map(|_| ProgramZone {
            pitch_finetuning: be_bytes!(mus_bytes),
            reverb: be_bytes!(mus_bytes),
            pan_position: float_be_bytes!(mus_bytes),
            keynum_hold: be_bytes!(mus_bytes),
            keynum_decay: be_bytes!(mus_bytes),
            volume_env: Envelope::parse(&mut mus_bytes),
            volume_env_atten: float_be_bytes!(mus_bytes),
            vib_delay: float_be_bytes!(mus_bytes),
            vib_frequency: float_be_bytes!(mus_bytes),
            vib_to_pitch: float_be_bytes!(mus_bytes),
            root_key: be_bytes!(mus_bytes),
            note_low: mus_bytes.next().unwrap(),
            note_high: mus_bytes.next().unwrap(),
            velocity_low: mus_bytes.next().unwrap(),
            velocity_high: mus_bytes.next().unwrap(),
            wave_index: be_bytes!(mus_bytes),
            base_priority: float_be_bytes!(mus_bytes),
            modul_env: Envelope::parse(&mut mus_bytes),
            modul_env_to_pitch: float_be_bytes!(mus_bytes),
        })
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&program_zones);
    }

    let preset_entries = (0..header.num_presets)
        .map(|_| PresetEntry {
            name: name_bytes!(mus_bytes),
            midi_bank_number: be_bytes!(mus_bytes),
            midi_preset_number: be_bytes!(mus_bytes),
            num_zones: be_bytes!(mus_bytes),
        })
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&preset_entries);
    }

    let preset_zones = (0..header.num_presets)
        .map(|_| PresetZone {
            root_key: be_bytes!(mus_bytes),
            note_low: mus_bytes.next().unwrap(),
            note_high: mus_bytes.next().unwrap(),
            velocity_low: mus_bytes.next().unwrap(),
            velocity_high: mus_bytes.next().unwrap(),
            program_index: be_bytes!(mus_bytes),
        })
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&preset_zones);
    }
}

#[derive(Debug)]
struct MusHeader {
    magic: u32,
    header_size: u32,
    version_number: u32,
    reverb_volume: u32,
    reverb_type: u32,
    reverb_multiply: u32,
    num_sequences: u32,
    num_labels: u32,
    offset_to_labels_offsets_table: u32,
    num_waves: u32,
    num_programs: u32,
    num_presets: u32,
}

#[derive(Debug)]
struct MsqTable {
    index: u32,
    offset: u32,
}

#[derive(Debug)]
struct WaveEntry {
    name: [char; 20],
    offset: u32,
    loop_begin: u32,
    size: u32,
    loop_end: u32,
    sample_rate: u32,
    original_pitch: u32,
    loop_info: u32,
    snd_handle: u32,
}

impl WaveEntry {
    fn valid_char(c: &u8) -> bool {
        match c {
            34 | 36 | 42 | 47 | 58 | 59 | 60 | 62 | 63 | 92 | 94 | 96 => false,
            32..=126 => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
struct Envelope {
    delay: f32,
    attack: f32,
    hold: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

impl Envelope {
    fn parse(bytes: &mut impl Iterator<Item = u8>) -> Self {
        Self {
            delay: float_be_bytes!(bytes),
            attack: float_be_bytes!(bytes),
            hold: float_be_bytes!(bytes),
            decay: float_be_bytes!(bytes),
            sustain: float_be_bytes!(bytes),
            release: float_be_bytes!(bytes),
        }
    }
}

#[derive(Debug)]
struct ProgramZone {
    pitch_finetuning: u32,
    reverb: u32,
    pan_position: f32,
    keynum_hold: u32,
    keynum_decay: u32,
    volume_env: Envelope,
    volume_env_atten: f32,
    vib_delay: f32,
    vib_frequency: f32,
    vib_to_pitch: f32,
    // usually padded as 0xFFFFFFFF. Copy the value from the "originalPitch" variable from the "waveEntry" structure */
    root_key: u32,
    note_low: u8,
    note_high: u8,
    velocity_low: u8,
    velocity_high: u8,
    wave_index: u32,
    base_priority: f32,
    modul_env: Envelope,
    modul_env_to_pitch: f32,
}

#[derive(Debug)]
struct ProgramEntry {
    name: [char; 20],
    num_zones: u32,
}

#[derive(Debug)]
struct PresetZone {
    root_key: u32,
    note_low: u8,
    note_high: u8,
    velocity_low: u8,
    velocity_high: u8,
    program_index: u32,
}

#[derive(Debug)]
struct PresetEntry {
    name: [char; 20],
    midi_bank_number: u32,
    midi_preset_number: u32,
    num_zones: u32,
}
