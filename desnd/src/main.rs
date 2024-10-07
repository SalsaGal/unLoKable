#![allow(dead_code)]

use std::{fmt::Display, fs::File, io::Write, ops::Range, path::PathBuf, slice::Iter};

use core::{
    clap::{self, Parser, ValueEnum},
    log::error,
};

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

fn four_bytes(bytes: &mut Iter<u8>) -> Option<[u8; 4]> {
    Some([
        *bytes.next()?,
        *bytes.next()?,
        *bytes.next()?,
        *bytes.next()?,
    ])
}

fn main() {
    core::init();

    let args = Args::parse();

    let snd_bytes = std::fs::read(&args.snd_path).unwrap_or_else(|e| {
        error!("Unable to load SND file {:?}: {e}", &args.snd_path);
        std::process::exit(1);
    });
    let smp_bytes = std::fs::read(&args.smp_path).unwrap_or_else(|e| {
        error!("Unable to load SMP file {:?}: {e}", &args.smp_path);
        std::process::exit(1);
    });

    let Some(snd_file) = SndFile::parse(
        &mut snd_bytes.iter(),
        snd_bytes.len() as u32,
        args.file_version.unwrap_or_default(),
    ) else {
        error!("Unable to parse SND file");
        std::process::exit(1);
    };
    let Some(smp_file) = SmpFile::parse(&snd_file, &mut smp_bytes.iter(), smp_bytes.len() as u32)
    else {
        error!("Unable to parse SMP file");
        std::process::exit(1);
    };

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

    // Because some zones can access more zones than are available, we have to skip them in a lot of things
    // The way we "skip" it is by setting the number of zones a program requests to zero.
    // This keeps the program around, but can also be a little confusing in some places (hence annoying iterator code like this)
    let finalised_number_of_programs =
        snd_file.programs.iter().filter(|p| p.num_zones > 0).count() as u32;
    vh_output
        .write_all(
            &[
                [0x70, 0x42, 0x41, 0x56],
                [7, 0, 0, 0],
                [0; 4],
                (32 + 2048
                    + finalised_number_of_programs * 512
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
                    finalised_number_of_programs.to_le_bytes()[0],
                    finalised_number_of_programs.to_le_bytes()[1],
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
    let mut bank_attribute: i16 = -1;
    let mut zone_index = 0;
    for program in &snd_file.programs {
        if let Some(zone) = snd_file.zones.get(zone_index) {
            if zone.parent_program == 0 {
                bank_attribute += 1;
            }
        }
        zone_index += program.num_zones as usize;
        vh_output
            .write_all(&[
                program.num_zones.to_ne_bytes()[0],
                program.volume,
                0,
                0,
                program.pan_pos,
                0,
                bank_attribute.to_ne_bytes()[0],
                bank_attribute.to_ne_bytes()[1],
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

    let mut zone_iter = snd_file.zones.iter();
    for program in &snd_file.programs {
        for _ in 0..program.num_zones {
            let zone = zone_iter.next().unwrap();
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
        }
        if program.num_zones > 0 {
            vh_output
                .write_all(
                    &std::iter::repeat(0)
                        .take(32 * (16 - program.num_zones as usize))
                        .collect::<Vec<_>>(),
                )
                .unwrap();
        }
    }
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

    if snd_file.header.num_labels != 0 {
        let label_path = output_folder
            .join(args.snd_path.with_extension("lbl").file_name().unwrap())
            .with_extension("lbl");
        let mut label_file = File::create(label_path).unwrap();
        label_file.write_all(&[0x4C, 0x42, 0x4C, 0x61]).unwrap(); // Header, LBLa
        label_file
            .write_all(&snd_file.header.num_labels.to_le_bytes())
            .unwrap();
        for label in snd_file.labels {
            label_file.write_all(&label.to_le_bytes()).unwrap();
        }
    }

    println!("SND header");
    println!("Header bytes: {}", snd_file.header.header_size);
    if let Some(version) = snd_file.header.bank_version {
        let minor = version.to_le_bytes()[0];
        let major = version.to_le_bytes()[1];
        println!(
            "SND version: {major}.{minor} ({:?})",
            args.file_version.unwrap_or_default()
        );
    } else {
        println!("SND version: Gex");
    }
    println!(
        "System: {}",
        if args.dreamcast {
            "Dreamcast"
        } else {
            "PlayStation"
        },
    );
    println!("Reverb mode: {}", snd_file.header.reverb_mode);
    println!("Reverb depth: {}", snd_file.header.reverb_depth);
    println!("Instruments: {}", snd_file.header.num_programs);
    println!("Zones: {}", snd_file.header.num_zones);
    println!("Samples: {}", snd_file.header.num_waves);
    println!("Sequences: {}", snd_file.header.num_sequences);
    println!("Labels: {}", snd_file.header.num_labels);
}

#[derive(Debug)]
struct HeaderSize {
    size: i64,
    original: Option<i64>,
}

impl HeaderSize {
    fn new(x: impl Into<i64>) -> Self {
        let x = x.into();
        if x % 4 == 0 {
            Self {
                size: x,
                original: None,
            }
        } else {
            Self {
                size: x - x % 4 + 4,
                original: Some(x),
            }
        }
    }
}

impl Display for HeaderSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.original {
            Some(original) => write!(f, "{original} ({}) bytes", self.size),
            None => write!(f, "{} bytes", self.size),
        }
    }
}

#[test]
fn rounding() {
    assert_eq!(HeaderSize::new(40).size, 40);
    assert_eq!(HeaderSize::new(41u8).size, 44);
    assert_eq!(HeaderSize::new(42u16).size, 44);
    assert_eq!(HeaderSize::new(43u32).size, 44);
    assert_eq!(HeaderSize::new(44).size, 44);
    assert_eq!(HeaderSize::new(45).size, 48);
}

#[derive(Debug)]
struct SndHeader {
    magic_number: u32,
    header_size: HeaderSize,
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
    fn parse(bytes: &mut Iter<u8>, version: Version) -> Option<Self> {
        Some(match version {
            Version::SoulReaver => Self {
                magic_number: u32::from_le_bytes(four_bytes(bytes)?),
                header_size: HeaderSize::new(u32::from_le_bytes(four_bytes(bytes)?)),
                bank_version: Some(u32::from_le_bytes(four_bytes(bytes)?)),
                num_programs: u32::from_le_bytes(four_bytes(bytes)?),
                num_zones: u32::from_le_bytes(four_bytes(bytes)?),
                num_waves: u32::from_le_bytes(four_bytes(bytes)?),
                num_sequences: u32::from_le_bytes(four_bytes(bytes)?),
                num_labels: u32::from_le_bytes(four_bytes(bytes)?),
                reverb_mode: u32::from_le_bytes(four_bytes(bytes)?),
                reverb_depth: u32::from_le_bytes(four_bytes(bytes)?),
            },
            Version::Prototype => Self {
                magic_number: u32::from_le_bytes(four_bytes(bytes)?),
                header_size: HeaderSize::new(u32::from_le_bytes(four_bytes(bytes)?)),
                bank_version: Some(u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32),
                num_programs: {
                    let _pad = bytes.next();
                    *bytes.next()? as u32
                },
                num_zones: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
                num_waves: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
                num_sequences: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
                num_labels: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
                reverb_mode: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
                reverb_depth: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
            },
            Version::Gex => Self {
                magic_number: u32::from_le_bytes(four_bytes(bytes)?),
                header_size: HeaderSize::new(u16::from_le_bytes([*bytes.next()?, *bytes.next()?])),
                bank_version: None,
                num_programs: {
                    let _pad = bytes.next();
                    // TODO try *bytes.nth(1)? as u32
                    // Honestly a lot of parts have repeated next calls that should be nth or skip
                    *bytes.next()? as u32
                },
                num_zones: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
                num_waves: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
                num_sequences: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
                num_labels: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
                reverb_mode: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
                reverb_depth: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]) as u32,
            },
        })
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
    fn parse(bytes: &mut Iter<u8>) -> Option<Self> {
        let program = Self {
            num_zones: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            first_tone: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            volume: *bytes.next()?,
            pan_pos: *bytes.next()?,
        };

        bytes.next()?;
        bytes.next()?;

        Some(program)
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
    fn parse(bytes: &mut Iter<u8>) -> Option<Self> {
        Some(Self {
            priority: *bytes.next()?,
            parent_program: *bytes.next()?,
            volume: *bytes.next()?,
            pan_pos: *bytes.next()?,
            root_key: *bytes.next()?,
            pitch_fine_tuning: *bytes.next()?,
            note_low: *bytes.next()?,
            note_high: *bytes.next()?,
            mode: *bytes.next()?,
            max_pitch_range: *bytes.next()?,
            adsr1: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            adsr2: u16::from_le_bytes([*bytes.next()?, *bytes.next()?]),
            wave_index: u16::from_le_bytes([*bytes.next()?, *bytes.next()?])
                .checked_add(1)
                .unwrap_or(1),
        })
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
    fn parse(bytes: &mut Iter<u8>, file_size: u32, version: Version) -> Option<Self> {
        let header = SndHeader::parse(bytes, version)?;
        assert_eq!(header.magic_number, 0x6153_4e44);

        while file_size - (bytes.as_slice().len() as u32) < header.header_size.size as u32 {
            bytes.next();
        }

        let mut requested_zones = 0;
        let programs = (0..header.num_programs)
            .map(|_| {
                let mut program = SndProgram::parse(bytes)?;
                if requested_zones + program.num_zones > header.num_zones as u16 {
                    program.num_zones = 0;
                } else {
                    requested_zones += program.num_zones;
                }
                Some(program)
            })
            .collect::<Option<Vec<_>>>()?;
        let zones = (0..header.num_zones)
            .map(|_| SndZone::parse(bytes))
            .collect::<Option<Vec<_>>>()?;
        let mut wave_offsets_start = None;
        let wave_offsets = (0..header.num_waves)
            .map(|_| {
                let num = u32::from_le_bytes(four_bytes(bytes)?);
                if wave_offsets_start.is_none() {
                    wave_offsets_start = Some(num);
                }
                Some(num - wave_offsets_start.unwrap())
            })
            .collect::<Option<Vec<_>>>()?;
        let sequence_offsets = (0..header.num_sequences)
            .map(|_| Some(u32::from_le_bytes(four_bytes(bytes)?)))
            .collect::<Option<Vec<_>>>()?;
        let labels = (0..header.num_labels)
            .map(|_| Some(u32::from_le_bytes(four_bytes(bytes)?)))
            .collect::<Option<Vec<_>>>()?;

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

        Some(Self {
            header,
            programs,
            zones,
            wave_offsets,
            sequence_offsets,
            labels,
            sequences,
        })
    }
}

#[derive(Debug)]
pub struct SmpFile {
    magic_number: Option<u32>,
    body_size: u32,
    waves: Vec<Range<u32>>,
}

impl SmpFile {
    fn parse(snd: &SndFile, bytes: &mut Iter<u8>, file_size: u32) -> Option<Self> {
        const MAGIC: u32 = 0x61534d50;
        let first_bytes = u32::from_le_bytes(four_bytes(bytes)?);

        let (magic_number, body_size, header_size) = if first_bytes == MAGIC {
            (Some(first_bytes), u32::from_le_bytes(four_bytes(bytes)?), 8)
        } else {
            (None, first_bytes, 4)
        };

        Some(Self {
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
        })
    }
}
