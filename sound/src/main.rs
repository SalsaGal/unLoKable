use std::{fs::File, io::Write, ops::Range, path::PathBuf, slice::Iter};

use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Args {
    snd_path: PathBuf,
    smp_path: PathBuf,
    #[clap(short, long)]
    dreamcast: bool,
    #[clap(short)]
    cents_tuning: bool,
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn four_bytes(bytes: &mut Iter<u8>) -> [u8; 4] {
    [
        *bytes.next().unwrap(),
        *bytes.next().unwrap(),
        *bytes.next().unwrap(),
        *bytes.next().unwrap(),
    ]
}

fn main() {
    let args = Args::parse();

    let snd_bytes = std::fs::read(&args.snd_path).unwrap();
    let smp_bytes = std::fs::read(&args.smp_path).unwrap();

    let snd_file = SndFile::parse(
        &mut snd_bytes.iter(),
        snd_bytes.len() as u32,
        args.cents_tuning,
    );
    let smp_file = SmpFile::parse(&snd_file, &mut smp_bytes.iter(), smp_bytes.len() as u32);

    let output_folder = args
        .output
        .unwrap_or_else(|| args.snd_path.with_extension(""));
    std::fs::create_dir(&output_folder).unwrap();
    let sequences_folder = output_folder.join("sequences");
    std::fs::create_dir(&sequences_folder).unwrap();
    let samples_folder = output_folder.join("samples");
    std::fs::create_dir(&samples_folder).unwrap();

    for (i, sequence) in snd_file.sequences.into_iter().enumerate() {
        let output_path = sequences_folder.join(format!(
            "{}_{i:04}.msq",
            output_folder.file_name().unwrap().to_string_lossy()
        ));

        let mut output_file = File::create(output_path).unwrap();
        let range = sequence.start as usize..sequence.end as usize;
        output_file.write_all(&snd_bytes[range]).unwrap();
    }

    for (i, wave) in smp_file.waves.into_iter().enumerate() {
        let output_path = samples_folder.join(format!(
            "{}_{i:04}.msq",
            output_folder.file_name().unwrap().to_string_lossy()
        ));

        let mut output_file = File::create(output_path).unwrap();
        let sample_length = wave.end - wave.start;
        if !args.dreamcast {
            output_file
                .write_all(
                    &[
                        [0x56, 0x41, 0x47, 0x70], // Magic number
                        [0, 0, 0, 3],             // Version number,
                        [0; 4],                   // Padding
                        sample_length.to_le_bytes(),
                        [0x00, 0x00, 0xAC, 0x44], // Sample rate
                        [0; 4],                   // Padding
                        [0; 4],
                        [0; 4],
                        [0; 4], // Name
                        [0; 4],
                        [0; 4],
                        [0; 4],
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<u8>>(),
                )
                .unwrap();
        }
        let range = wave.start as usize..wave.end as usize;
        output_file.write_all(&smp_bytes[range]).unwrap();
    }
}

#[derive(Debug)]
struct SndHeader {
    magic_number: u32,
    header_size: i32,
    bank_version: i32,
    num_programs: i32,
    num_zones: i32,
    num_waves: i32,
    num_sequences: i32,
    num_labels: i32,
    reverb_mode: i32,
    reverb_depth: i32,
}

impl SndHeader {
    fn parse(bytes: &mut Iter<u8>) -> Self {
        Self {
            magic_number: u32::from_le_bytes(four_bytes(bytes)),
            header_size: i32::from_le_bytes(four_bytes(bytes)),
            bank_version: i32::from_le_bytes(four_bytes(bytes)),
            num_programs: i32::from_le_bytes(four_bytes(bytes)),
            num_zones: i32::from_le_bytes(four_bytes(bytes)),
            num_waves: i32::from_le_bytes(four_bytes(bytes)),
            num_sequences: i32::from_le_bytes(four_bytes(bytes)),
            num_labels: i32::from_le_bytes(four_bytes(bytes)),
            reverb_mode: i32::from_le_bytes(four_bytes(bytes)),
            reverb_depth: i32::from_le_bytes(four_bytes(bytes)),
        }
    }
}

#[derive(Debug)]
struct SndProgram {
    num_zones: u16,
    first_tone: u16,
    volume: u8,
    pan_pos: u8,
}

impl SndProgram {
    fn parse(bytes: &mut Iter<u8>) -> Self {
        Self {
            num_zones: u16::from_be_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            first_tone: u16::from_be_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            volume: *bytes.next().unwrap(),
            pan_pos: *bytes.next().unwrap(),
        }
    }
}

#[derive(Debug)]
struct SndZone {
    priority: u8,
    parent_program: u8,
    volume: u8,
    pan_pos: u8,
    root_key: u8,
    pitch_fine_tuning: u8,
    note_low: u8,
    note_high: u8,
    node: u8,
    max_pitch_range: u8,
    adsr1: u16,
    adsr2: u16,
    wave_index: u16,
}

impl SndZone {
    fn parse(bytes: &mut Iter<u8>, cents_tuning: bool) -> Self {
        Self {
            priority: *bytes.next().unwrap(),
            parent_program: *bytes.next().unwrap(),
            volume: *bytes.next().unwrap(),
            pan_pos: *bytes.next().unwrap(),
            root_key: *bytes.next().unwrap(),
            pitch_fine_tuning: if cents_tuning {
                ((*bytes.next().unwrap() as f32) * 100.0 / 128.0) as u8
            } else {
                *bytes.next().unwrap()
            },
            note_low: *bytes.next().unwrap(),
            note_high: *bytes.next().unwrap(),
            node: *bytes.next().unwrap(),
            max_pitch_range: *bytes.next().unwrap(),
            adsr1: u16::from_be_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            adsr2: u16::from_be_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            wave_index: u16::from_be_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
        }
    }
}

#[derive(Debug)]
struct SndFile {
    header: SndHeader,
    programs: Vec<SndProgram>,
    zones: Vec<SndZone>,
    wave_offsets: Vec<u32>,
    sequence_offsets: Vec<u32>,
    labels: Vec<u32>,
    sequences: Vec<Range<u32>>,
}

impl SndFile {
    fn parse(bytes: &mut Iter<u8>, file_size: u32, cents_tuning: bool) -> Self {
        let header = SndHeader::parse(bytes);
        assert_eq!(header.magic_number, 0x61534e44);

        let programs = (0..header.num_programs)
            .map(|_| SndProgram::parse(bytes))
            .collect();
        let zones = (0..header.num_zones)
            .map(|_| SndZone::parse(bytes, cents_tuning))
            .collect();
        let wave_offsets = (0..header.num_waves)
            .map(|_| u32::from_le_bytes(four_bytes(bytes)))
            .collect();
        let sequence_offsets = (0..header.num_sequences)
            .map(|_| u32::from_le_bytes(four_bytes(bytes)))
            .collect::<Vec<_>>();
        let labels = (0..header.num_labels)
            .map(|_| u32::from_le_bytes(four_bytes(bytes)))
            .collect();

        let mut sequences = vec![];
        for i in (0..header.num_sequences).map(|i| i as usize) {
            let start = sequence_offsets[i];
            let end = if i == header.num_sequences as usize - 1 {
                file_size
            } else {
                sequence_offsets[i + 1]
            };
            sequences.push(start..end);
        }

        Self {
            programs,
            zones,
            wave_offsets,
            sequence_offsets,
            labels,
            sequences,
            header,
        }
    }
}

#[derive(Debug)]
pub struct SmpFile {
    magic_number: u32,
    body_size: u32,
    waves: Vec<Range<u32>>,
}

impl SmpFile {
    fn parse(snd: &SndFile, bytes: &mut Iter<u8>, file_size: u32) -> Self {
        Self {
            magic_number: u32::from_le_bytes(four_bytes(bytes)),
            body_size: u32::from_le_bytes(four_bytes(bytes)),
            waves: (0..snd.header.num_waves as usize)
                .map(|i| {
                    let start = snd.wave_offsets[i];
                    let end = if i == snd.header.num_waves as usize - 1 {
                        file_size
                    } else {
                        snd.wave_offsets[i + 1]
                    };
                    start..end
                })
                .collect(),
        }
    }
}
