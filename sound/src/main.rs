use std::{fs::File, io::Write, ops::Range, path::PathBuf, slice::Iter};

use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum Version {
    #[default]
    SoulReaver,
    Prototype,
    Gex,
}

#[derive(Parser)]
#[command(version)]
struct Args {
    /// The `snd` path to load from.
    snd_path: PathBuf,
    /// The `smp` path to load from.
    smp_path: PathBuf,
    /// What version the `snd` file is.
    #[clap(short)]
    file_version: Option<Version>,
    /// Whether on the Dreamcast platform or not.
    #[clap(short, long)]
    dreamcast: bool,
    /// Folder to put output files in.
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn align(x: impl Into<i64>) -> i64 {
    let x = x.into();
    if x % 4 == 0 {
        x
    } else {
        x - x % 4 + 4
    }
}

#[test]
fn rounding() {
    assert_eq!(align(40), 40);
    assert_eq!(align(41u8), 44);
    assert_eq!(align(42u16), 44);
    assert_eq!(align(43u32), 44);
    assert_eq!(align(44), 44);
    assert_eq!(align(45), 48);
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
        args.file_version.unwrap_or_default(),
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
        let range = sequence.start as usize..sequence.end as usize;
        let bytes = &snd_bytes[range];
        let extension = match bytes[0..4] {
            [0x51, 0x53, 0x4d, 0x61] => "msq",
            [0x51, 0x45, 0x53, 0x61] => "cds",
            _ => panic!(
                "Unsupported sequence magic number, {}_{i:04}",
                output_folder.file_name().unwrap().to_string_lossy()
            ),
        };

        let output_path = sequences_folder.join(format!(
            "{}_{i:04}.{extension}",
            output_folder.file_name().unwrap().to_string_lossy()
        ));
        let mut output_file = File::create(output_path).unwrap();
        output_file.write_all(bytes).unwrap();
    }

    for (i, wave) in smp_file.waves.iter().enumerate() {
        let output_path = samples_folder.join(format!(
            "{}_{i:04}.{}",
            output_folder.file_name().unwrap().to_string_lossy(),
            if args.dreamcast { "dcs" } else { "vag" }
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
                        sample_length.to_be_bytes(),
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

    let vh_output_path = output_folder.join(
        PathBuf::from(
            output_folder
                .file_name()
                .unwrap()
                .to_string_lossy()
                .as_ref(),
        )
        .with_extension("vh"),
    );
    let mut vh_output = File::create(vh_output_path).unwrap();
    vh_output
        .write_all(
            &[
                [0x70, 0x42, 0x41, 0x56],
                [7, 0, 0, 0],
                [0; 4],
                (32 + 2048
                    + snd_file.header.num_programs * 512
                    + 512
                    + smp_file
                        .waves
                        .iter()
                        .map(|range| range.end - range.start)
                        .sum::<u32>())
                .to_le_bytes(),
                [
                    0,
                    0,
                    snd_file.header.num_programs.to_le_bytes()[0],
                    snd_file.header.num_programs.to_le_bytes()[1],
                ],
                [
                    snd_file.header.num_zones.to_le_bytes()[0],
                    snd_file.header.num_zones.to_le_bytes()[1],
                    snd_file.header.num_waves.to_le_bytes()[0],
                    snd_file.header.num_waves.to_le_bytes()[1],
                ],
                [0x7f, 0x40, 0, 0],
                [0; 4],
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        )
        .unwrap();
    for program in &snd_file.programs {
        vh_output
            .write_all(&[
                (program.num_zones >> 8) as u8,
                program.volume,
                0,
                0,
                program.pan_pos,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ])
            .unwrap();
    }
    vh_output
        .write_all(
            &std::iter::repeat(0)
                .take(16 * (128 - snd_file.header.num_programs as usize))
                .collect::<Vec<_>>(),
        )
        .unwrap();

    let vb_output_path = output_folder.join(
        PathBuf::from(
            output_folder
                .file_name()
                .unwrap()
                .to_string_lossy()
                .as_ref(),
        )
        .with_extension("vb"),
    );
    let mut vb_output = File::create(vb_output_path).unwrap();
    for waves in &smp_file.waves {
        vb_output
            .write_all(&smp_bytes[waves.start as usize..waves.end as usize])
            .unwrap();
    }

    let mut current_parent_program = 0;
    let mut current_parent_program_streak = 0;
    for zone in &snd_file.zones {
        if zone.parent_program != current_parent_program {
            vh_output
                .write_all(
                    &std::iter::repeat(0)
                        .take(32 * (16 - current_parent_program_streak))
                        .collect::<Vec<_>>(),
                )
                .unwrap();
            current_parent_program = zone.parent_program;
            current_parent_program_streak = 0;
        }

        vh_output
            .write_all(
                &[
                    [zone.priority, zone.mode],
                    [zone.volume, zone.pan_pos],
                    [zone.root_key, zone.pitch_fine_tuning],
                    [zone.note_low, zone.note_high],
                    [0; 2],
                    [0; 2],
                    [zone.max_pitch_range, zone.max_pitch_range],
                    [0; 2],
                    zone.adsr1.to_le_bytes(),
                    zone.adsr2.to_le_bytes(),
                    [zone.parent_program, 0],
                    zone.wave_index.to_le_bytes(),
                    [0; 2],
                    [0; 2],
                    [0; 2],
                    [0; 2],
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<u8>>(),
            )
            .unwrap();
        current_parent_program_streak += 1;
    }
    vh_output
        .write_all(
            &std::iter::repeat(0)
                .take(32 * (16 - current_parent_program_streak))
                .collect::<Vec<_>>(),
        )
        .unwrap();
    vh_output.write_all(&[0; 2]).unwrap();
    for wave in &smp_file.waves {
        let size = (wave.end - wave.start) / 8;
        let size = (size as u16).to_le_bytes();
        vh_output.write_all(&size).unwrap();
    }
    vh_output
        .write_all(
            &std::iter::repeat(0)
                .take(512 - (1 + smp_file.waves.len()) * 2)
                .collect::<Vec<_>>(),
        )
        .unwrap();
}

#[derive(Debug)]
struct SndHeader {
    magic_number: u32,
    header_size: u32,
    bank_version: Option<u32>,
    num_programs: u32,
    num_zones: u32,
    num_waves: u32,
    num_sequences: u32,
    num_labels: u32,
    reverb_mode: u32,
    reverb_depth: u32,
}

impl SndHeader {
    fn parse(bytes: &mut Iter<u8>, version: Version) -> Self {
        match version {
            Version::SoulReaver => Self {
                magic_number: u32::from_le_bytes(four_bytes(bytes)),
                header_size: align(u32::from_le_bytes(four_bytes(bytes))) as u32,
                bank_version: Some(u32::from_le_bytes(four_bytes(bytes))),
                num_programs: u32::from_le_bytes(four_bytes(bytes)),
                num_zones: u32::from_le_bytes(four_bytes(bytes)),
                num_waves: u32::from_le_bytes(four_bytes(bytes)),
                num_sequences: u32::from_le_bytes(four_bytes(bytes)),
                num_labels: u32::from_le_bytes(four_bytes(bytes)),
                reverb_mode: u32::from_le_bytes(four_bytes(bytes)),
                reverb_depth: u32::from_le_bytes(four_bytes(bytes)),
            },
            Version::Prototype => Self {
                magic_number: u32::from_le_bytes(four_bytes(bytes)),
                header_size: align(u32::from_le_bytes(four_bytes(bytes))) as u32,
                bank_version: Some(u16::from_le_bytes([
                    *bytes.next().unwrap(),
                    *bytes.next().unwrap(),
                ]) as u32),
                num_programs: {
                    let _pad = bytes.next();
                    *bytes.next().unwrap() as u32
                },
                num_zones: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
                num_waves: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
                num_sequences: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
                num_labels: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
                reverb_mode: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
                reverb_depth: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
            },
            Version::Gex => Self {
                magic_number: u32::from_le_bytes(four_bytes(bytes)),
                header_size: align(u16::from_le_bytes([
                    *bytes.next().unwrap(),
                    *bytes.next().unwrap(),
                ])) as u32,
                bank_version: None,
                num_programs: {
                    let _pad = bytes.next();
                    *bytes.next().unwrap() as u32
                },
                num_zones: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
                num_waves: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
                num_sequences: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
                num_labels: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
                reverb_mode: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
                reverb_depth: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                    as u32,
            },
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
        let program = Self {
            num_zones: u16::from_be_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            first_tone: u16::from_be_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            volume: *bytes.next().unwrap(),
            pan_pos: *bytes.next().unwrap(),
        };

        bytes.next().unwrap();
        bytes.next().unwrap();

        program
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
    mode: u8,
    max_pitch_range: u8,
    adsr1: u16,
    adsr2: u16,
    wave_index: u16,
}

impl SndZone {
    fn parse(bytes: &mut Iter<u8>) -> Self {
        Self {
            priority: *bytes.next().unwrap(),
            parent_program: *bytes.next().unwrap(),
            volume: *bytes.next().unwrap(),
            pan_pos: *bytes.next().unwrap(),
            root_key: *bytes.next().unwrap(),
            pitch_fine_tuning: *bytes.next().unwrap(),
            note_low: *bytes.next().unwrap(),
            note_high: *bytes.next().unwrap(),
            mode: *bytes.next().unwrap(),
            max_pitch_range: *bytes.next().unwrap(),
            adsr1: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            adsr2: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()]),
            wave_index: u16::from_le_bytes([*bytes.next().unwrap(), *bytes.next().unwrap()])
                .checked_add(1)
                .unwrap_or(1),
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
    fn parse(bytes: &mut Iter<u8>, file_size: u32, version: Version) -> Self {
        let header = SndHeader::parse(bytes, version);
        assert_eq!(header.magic_number, 0x6153_4e44);

        while file_size - (bytes.as_slice().len() as u32) < header.header_size as u32 {
            bytes.next();
        }

        let programs = (0..header.num_programs)
            .map(|_| SndProgram::parse(bytes))
            .collect();
        let zones = (0..header.num_zones)
            .map(|_| SndZone::parse(bytes))
            .collect();
        let mut wave_offsets_start = None;
        let wave_offsets = (0..header.num_waves)
            .map(|_| {
                let num = u32::from_le_bytes(four_bytes(bytes));
                if wave_offsets_start.is_none() {
                    wave_offsets_start = Some(num);
                }
                num - wave_offsets_start.unwrap()
            })
            .collect();
        let sequence_offsets = (0..header.num_sequences)
            .map(|_| u32::from_le_bytes(four_bytes(bytes)))
            .collect::<Vec<_>>();
        let labels = (0..header.num_labels)
            .map(|_| u32::from_le_bytes(four_bytes(bytes)))
            .collect();

        let sequences_start = file_size - bytes.as_slice().len() as u32;
        let mut sequences = vec![];
        for i in (0..header.num_sequences).map(|i| i as usize) {
            let start = sequences_start + sequence_offsets[i];
            let end = if i == header.num_sequences as usize - 1 {
                file_size
            } else {
                sequences_start + sequence_offsets[i + 1]
            };
            sequences.push(start..end);
        }

        Self {
            header,
            programs,
            zones,
            wave_offsets,
            sequence_offsets,
            labels,
            sequences,
        }
    }
}

#[derive(Debug)]
pub struct SmpFile {
    magic_number: Option<u32>,
    body_size: u32,
    waves: Vec<Range<u32>>,
}

impl SmpFile {
    fn parse(snd: &SndFile, bytes: &mut Iter<u8>, file_size: u32) -> Self {
        const MAGIC: u32 = 0x61534d50;
        let first_bytes = u32::from_le_bytes(four_bytes(bytes));

        let (magic_number, body_size, header_size) = if first_bytes == MAGIC {
            (Some(first_bytes), u32::from_le_bytes(four_bytes(bytes)), 8)
        } else {
            (None, first_bytes, 4)
        };

        Self {
            magic_number,
            body_size,
            waves: (0..snd.header.num_waves as usize)
                .map(|i| {
                    let start = header_size + snd.wave_offsets[i];
                    let end = if i == snd.header.num_waves as usize - 1 {
                        file_size
                    } else {
                        header_size + snd.wave_offsets[i + 1]
                    };
                    start..end
                })
                .collect(),
        }
    }
}
