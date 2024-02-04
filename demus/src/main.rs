use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

#[derive(Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Platform {
    Dreamcast,
    #[default]
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
    platform: Option<Platform>,
    /// Output path of the cds file, defaults to the input with a different extension
    #[clap(long, short)]
    output: Option<PathBuf>,
}

fn secs_to_timecent(seconds: f32) -> i32 {
    (1200.0 * f32::log2(seconds.max(0.001))) as i32
}

fn semitone_tuning(note: i32) -> i32 {
    note / 256
}

fn cents_tuning(note: i32) -> i32 {
    (note % 256) * 100 / 256
}

fn pan_convert(pan: f32) -> i32 {
    (pan * 1000.0 - 500.0) as i32
}

fn percentage_to_decibels(sustain: f32) -> i32 {
    (sustain * 10.0) as i32
}

macro_rules! le_bytes {
    ($bytes: ident) => {
        i32::from_le_bytes([
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
        ])
    };
}

macro_rules! be_bytes {
    ($bytes: ident) => {
        i32::from_be_bytes([
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
        ])
    };
}

macro_rules! float_le_bytes {
    ($bytes: ident) => {
        f32::from_le_bytes([
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
            $bytes.next().unwrap(),
        ])
    };
}

fn parse_name(bytes: &mut impl Iterator<Item = u8>) -> [char; 20] {
    let mut encountered_garbage = false;
    let mut bytes = ['\0'; 20].map(move |_| {
        let c = bytes.next().unwrap();
        // This first term isn't needed right?
        if !encountered_garbage && !WaveEntry::valid_char(&c) {
            encountered_garbage = true;
        }
        if encountered_garbage {
            '\0'
        } else {
            c.into()
        }
    });
    for c in bytes.iter_mut().rev() {
        if *c == ' ' {
            *c = '\0';
        } else if WaveEntry::valid_char(&(*c as u8)) {
            break;
        }
    }

    bytes
}

#[test]
fn test_parse_name() {
    assert_eq!(
        parse_name(&mut "C Hit          \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".bytes())[6],
        '\0'
    );
    assert_eq!(
        parse_name(&mut "C Hit\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".bytes())[2],
        'H'
    );
}

fn name_to_str(name: &[char; 20]) -> String {
    name.iter().take_while(|x| **x != '\0').collect()
}

fn main() {
    let args = Args::parse();

    let mus_file = std::fs::read(&args.mus_path).unwrap();
    let mut mus_bytes = mus_file.iter().copied();
    let mut sam_file = std::fs::read(&args.sam_path).unwrap();

    let header = MusHeader {
        magic: be_bytes!(mus_bytes),
        header_size: le_bytes!(mus_bytes),
        version_number: le_bytes!(mus_bytes),
        reverb_volume: le_bytes!(mus_bytes),
        reverb_type: le_bytes!(mus_bytes),
        reverb_multiply: le_bytes!(mus_bytes),
        num_sequences: le_bytes!(mus_bytes),
        num_labels: le_bytes!(mus_bytes),
        offset_to_labels_offsets_table: le_bytes!(mus_bytes),
        num_waves: le_bytes!(mus_bytes),
        num_programs: le_bytes!(mus_bytes),
        num_presets: le_bytes!(mus_bytes),
    };
    assert_eq!(header.magic, 0x4D757321, "Invalid magic number");
    if args.debug {
        dbg!(&header);
    }

    let msq_tables = (0..header.num_sequences)
        .map(|_| MsqTable {
            index: le_bytes!(mus_bytes),
            offset: le_bytes!(mus_bytes),
        })
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&msq_tables);
    }

    let layers = (0..header.num_presets + header.num_programs)
        .map(|_| le_bytes!(mus_bytes))
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&layers);
    }

    let wave_entries = (0..header.num_waves)
        .map(|_| WaveEntry {
            name: parse_name(&mut mus_bytes),
            offset: le_bytes!(mus_bytes),
            loop_begin: le_bytes!(mus_bytes),
            size: le_bytes!(mus_bytes) * 2,
            loop_end: le_bytes!(mus_bytes),
            sample_rate: le_bytes!(mus_bytes),
            original_pitch: le_bytes!(mus_bytes) >> 8,
            loop_info: le_bytes!(mus_bytes),
            snd_handle: le_bytes!(mus_bytes),
        })
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&wave_entries);
    }

    let mut program_entries = Vec::with_capacity(header.num_programs as usize);
    let mut program_zones = Vec::with_capacity(header.num_programs as usize);
    for _ in 0..header.num_programs {
        program_entries.push(ProgramEntry {
            name: parse_name(&mut mus_bytes),
            num_zones: le_bytes!(mus_bytes),
        });
        program_zones.push(Vec::with_capacity(
            program_entries.last().unwrap().num_zones as usize,
        ));
        for _ in 0..program_entries.last().unwrap().num_zones {
            program_zones.last_mut().unwrap().push(ProgramZone {
                pitch_finetuning: le_bytes!(mus_bytes),
                reverb: le_bytes!(mus_bytes),
                pan_position: float_le_bytes!(mus_bytes),
                keynum_hold: le_bytes!(mus_bytes),
                keynum_decay: le_bytes!(mus_bytes),
                volume_env: Envelope::parse(&mut mus_bytes),
                volume_env_atten: float_le_bytes!(mus_bytes),
                vib_delay: float_le_bytes!(mus_bytes),
                vib_frequency: float_le_bytes!(mus_bytes),
                vib_to_pitch: float_le_bytes!(mus_bytes),
                root_key: le_bytes!(mus_bytes),
                note_low: mus_bytes.next().unwrap(),
                note_high: mus_bytes.next().unwrap(),
                velocity_low: mus_bytes.next().unwrap(),
                velocity_high: mus_bytes.next().unwrap(),
                wave_index: le_bytes!(mus_bytes),
                base_priority: float_le_bytes!(mus_bytes),
                modul_env: Envelope::parse(&mut mus_bytes),
                modul_env_to_pitch: float_le_bytes!(mus_bytes),
            });
        }
    }
    if args.debug {
        dbg!(&program_entries, &program_zones);
    }

    let mut preset_entries = Vec::with_capacity(header.num_presets as usize);
    let mut preset_zones = Vec::with_capacity(header.num_presets as usize);
    for _ in 0..header.num_presets {
        preset_entries.push(PresetEntry {
            name: parse_name(&mut mus_bytes),
            midi_bank_number: le_bytes!(mus_bytes),
            midi_preset_number: le_bytes!(mus_bytes),
            num_zones: le_bytes!(mus_bytes),
        });
        dbg!(&preset_entries);
        preset_zones.push(Vec::with_capacity(
            preset_entries.last().unwrap().num_zones as usize,
        ));
        for _ in 0..preset_entries.last().unwrap().num_zones {
            preset_zones.last_mut().unwrap().push(PresetZone {
                root_key: le_bytes!(mus_bytes),
                note_low: mus_bytes.next().unwrap(),
                note_high: mus_bytes.next().unwrap(),
                velocity_low: mus_bytes.next().unwrap(),
                velocity_high: mus_bytes.next().unwrap(),
                program_index: le_bytes!(mus_bytes),
            });
        }
    }
    if args.debug {
        dbg!(&preset_entries, &preset_zones);
    }

    // Definitely doable with iterators, but too lazy to work it out right now
    let mut sequences: Vec<(i32, Option<i32>)> = Vec::with_capacity(header.num_sequences as usize);
    for i in 0..header.num_sequences as usize {
        sequences.push((msq_tables[i].offset, None));
        if i > 0 {
            sequences[i - 1].1 = Some(sequences[i].0 - sequences[i - 1].0);
        }
    }
    // Convert the ranges into actual slices
    let sequences = sequences
        .into_iter()
        .map(|(start, end)| match end {
            Some(end) => &mus_file[start as usize..start as usize + end as usize],
            None => &mus_file[start as usize..],
        })
        .collect::<Vec<_>>();
    if args.debug {
        dbg!(&sequences);
    }

    let sequences_dir = args.mus_path.with_extension("").join("sequences");
    std::fs::create_dir_all(&sequences_dir).unwrap();
    for (i, sequence) in sequences.into_iter().enumerate() {
        let path = sequences_dir.join(format!(
            "{}_{:04}.msq",
            args.mus_path.file_stem().unwrap().to_string_lossy(),
            i,
        ));
        let mut file = File::create(path).unwrap();
        file.write_all(sequence).unwrap();
    }

    let waves = wave_entries
        .iter()
        .map(|wave_entry| {
            dbg!(wave_entry);
            let wave_range =
                wave_entry.offset as usize..wave_entry.offset as usize + wave_entry.size as usize;
            if args.platform.unwrap_or_default() == Platform::Dreamcast {
                let check_index = wave_range.start + wave_range.end - 16;
                if sam_file[check_index..check_index + 16]
                    == [
                        0x07, 0x00, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77,
                        0x77, 0x77, 0x77, 0x77,
                    ]
                {
                    sam_file[check_index + 1] = 0x07;
                }
            }
            wave_range
        })
        .collect::<Vec<_>>();

    let samples_path = args.mus_path.with_extension("").join("samples");
    std::fs::create_dir(&samples_path).unwrap();

    for (wave, wave_entry) in waves.iter().zip(&wave_entries) {
        let path = samples_path.join(format!("{}.ads", name_to_str(&wave_entry.name)));
        let mut sample_file = File::create(path).unwrap();

        sample_file
            .write_all(&[0x53, 0x53, 0x68, 0x64, 0x18, 0x0, 0x0, 0x0])
            .unwrap();
        if args.platform.unwrap_or_default() == Platform::PC {
            sample_file.write_all(&[0x01]).unwrap();
        } else {
            sample_file.write_all(&[0x10]).unwrap();
        }
        sample_file.write_all(&[0; 3]).unwrap();
        sample_file
            .write_all(&wave_entry.sample_rate.to_be_bytes())
            .unwrap();
        sample_file.write_all(&[1]).unwrap();
        sample_file.write_all(&[0; 3]).unwrap();
        sample_file.write_all(&[0; 4]).unwrap();
        sample_file.write_all(&[0xff; 8]).unwrap();
        sample_file.write_all(&[0x53, 0x53, 0x62, 0x64]).unwrap();
        sample_file
            .write_all(&wave_entry.size.to_be_bytes())
            .unwrap();
        sample_file.write_all(&sam_file[wave.clone()]).unwrap();
    }

    let smp_loop_info_path = args.mus_path.with_extension("").join(format!(
        "{}_smploopinfo.txt",
        args.mus_path
            .with_extension("")
            .file_stem()
            .unwrap()
            .to_string_lossy()
    ));
    let mut smp_loop_info = File::create(smp_loop_info_path).unwrap();
    for entry in &wave_entries {
        smp_loop_info
            .write_all(
                format!(
                    "{} {} {}.wav\n",
                    entry.loop_begin,
                    entry.loop_end,
                    name_to_str(&entry.name),
                )
                .as_bytes(),
            )
            .unwrap()
    }
}

#[derive(Debug)]
struct MusHeader {
    magic: i32,
    header_size: i32,
    version_number: i32,
    reverb_volume: i32,
    reverb_type: i32,
    reverb_multiply: i32,
    num_sequences: i32,
    num_labels: i32,
    offset_to_labels_offsets_table: i32,
    num_waves: i32,
    num_programs: i32,
    num_presets: i32,
}

#[derive(Debug)]
struct MsqTable {
    index: i32,
    offset: i32,
}

#[derive(Debug)]
struct WaveEntry {
    name: [char; 20],
    offset: i32,
    loop_begin: i32,
    size: i32,
    loop_end: i32,
    sample_rate: i32,
    original_pitch: i32,
    loop_info: i32,
    snd_handle: i32,
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
            delay: float_le_bytes!(bytes),
            attack: float_le_bytes!(bytes),
            hold: float_le_bytes!(bytes),
            decay: float_le_bytes!(bytes),
            sustain: float_le_bytes!(bytes),
            release: float_le_bytes!(bytes),
        }
    }
}

#[derive(Debug)]
struct ProgramZone {
    pitch_finetuning: i32,
    reverb: i32,
    pan_position: f32,
    keynum_hold: i32,
    keynum_decay: i32,
    volume_env: Envelope,
    volume_env_atten: f32,
    vib_delay: f32,
    vib_frequency: f32,
    vib_to_pitch: f32,
    // usually padded as 0xFFFFFFFF. Copy the value from the "originalPitch" variable from the "waveEntry" structure */
    root_key: i32,
    note_low: u8,
    note_high: u8,
    velocity_low: u8,
    velocity_high: u8,
    wave_index: i32,
    base_priority: f32,
    modul_env: Envelope,
    modul_env_to_pitch: f32,
}

#[derive(Debug)]
struct ProgramEntry {
    name: [char; 20],
    num_zones: i32,
}

#[derive(Debug)]
struct PresetZone {
    root_key: i32,
    note_low: u8,
    note_high: u8,
    velocity_low: u8,
    velocity_high: u8,
    program_index: i32,
}

#[derive(Debug)]
struct PresetEntry {
    name: [char; 20],
    midi_bank_number: i32,
    midi_preset_number: i32,
    num_zones: i32,
}
