#![allow(dead_code)]

use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

const HEADER_VERSION_114: i32 = 270;
const HEADER_VERSION_120: i32 = 276;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Platform {
    Console,
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
    /// Tells program to use PC format. This is the default.
    #[clap(long, short)]
    pc: bool,
    /// Tells program to use console format.
    #[clap(long, short)]
    console: bool,
    /// Output folder of the files, defaults to the input with a different extension.
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
    let platform = if args.console {
        Platform::Console
    } else {
        Platform::PC
    };

    let mus_file = std::fs::read(&args.mus_path).unwrap();
    let mut mus_bytes = mus_file.iter().copied();
    let mut sam_file = std::fs::read(&args.sam_path).unwrap();

    let header = MusHeader::parse(&mut mus_bytes);
    assert_eq!(header.magic, 0x4D75_7321, "Invalid magic number");
    #[cfg(debug_assertions)]
    dbg!(&header);

    let msq_tables = (0..header.num_sequences)
        .map(|_| MsqTable {
            index: le_bytes!(mus_bytes),
            offset: le_bytes!(mus_bytes),
        })
        .collect::<Vec<_>>();
    #[cfg(debug_assertions)]
    dbg!(&msq_tables);

    let layers = (0..header.num_presets + header.num_programs)
        .map(|_| le_bytes!(mus_bytes))
        .collect::<Vec<_>>();
    #[cfg(debug_assertions)]
    dbg!(&layers);

    let wave_entries = (0..header.num_waves)
        .map(|_| WaveEntry {
            name: parse_name(&mut mus_bytes),
            offset: le_bytes!(mus_bytes),
            loop_begin: le_bytes!(mus_bytes),
            size: le_bytes!(mus_bytes) * 2,
            loop_end: le_bytes!(mus_bytes),
            sample_rate: le_bytes!(mus_bytes),
            original_pitch: le_bytes!(mus_bytes),
            loop_info: le_bytes!(mus_bytes),
            snd_handle: le_bytes!(mus_bytes),
        })
        .collect::<Vec<_>>();
    #[cfg(debug_assertions)]
    dbg!(&wave_entries);

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
    #[cfg(debug_assertions)]
    dbg!(&program_entries, &program_zones);

    let mut preset_entries = Vec::with_capacity(header.num_presets as usize);
    let mut preset_zones = Vec::with_capacity(header.num_presets as usize);
    for _ in 0..header.num_presets {
        preset_entries.push(PresetEntry {
            name: parse_name(&mut mus_bytes),
            midi_bank_number: le_bytes!(mus_bytes),
            midi_preset_number: le_bytes!(mus_bytes),
            num_zones: le_bytes!(mus_bytes),
        });
        #[cfg(debug_assertions)]
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
    #[cfg(debug_assertions)]
    dbg!(&preset_entries, &preset_zones);

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
            None => &mus_file[start as usize..header.offset_to_labels_offsets_table as usize],
        })
        .collect::<Vec<_>>();
    #[cfg(debug_assertions)]
    dbg!(&sequences);

    let sequences_dir = args.mus_path.with_extension("").join("sequences");
    let samples_dir = args.mus_path.with_extension("").join("samples");
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
            let wave_range =
                wave_entry.offset as usize..wave_entry.offset as usize + wave_entry.size as usize;
            if platform == Platform::Console {
                let check_index = wave_range.end - 16;
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

    std::fs::create_dir(&samples_dir).unwrap();
    for (wave, wave_entry) in waves.iter().zip(&wave_entries) {
        let path = samples_dir.join(format!("{}.ads", name_to_str(&wave_entry.name)));
        let mut sample_file = File::create(path).unwrap();

        sample_file
            .write_all(&[0x53, 0x53, 0x68, 0x64, 0x18, 0x0, 0x0, 0x0])
            .unwrap();
        if platform == Platform::PC {
            sample_file.write_all(&[0x01]).unwrap();
        } else {
            sample_file.write_all(&[0x10]).unwrap();
        }
        sample_file.write_all(&[0; 3]).unwrap();
        sample_file
            .write_all(&wave_entry.sample_rate.to_le_bytes())
            .unwrap();
        sample_file.write_all(&[1]).unwrap();
        sample_file.write_all(&[0; 3]).unwrap();
        sample_file.write_all(&[0; 4]).unwrap();
        sample_file.write_all(&[0xff; 8]).unwrap();
        sample_file.write_all(&[0x53, 0x53, 0x62, 0x64]).unwrap();
        sample_file
            .write_all(&wave_entry.size.to_le_bytes())
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
        if entry.loop_info != 0 {
            smp_loop_info
                .write_all(
                    format!(
                        "{} {} {}.wav\r\n",
                        entry.loop_begin,
                        entry.loop_end - 1,
                        name_to_str(&entry.name),
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
    }

    let info_path = args.mus_path.with_extension("").join(format!(
        "{}.txt",
        args.mus_path
            .with_extension("")
            .file_stem()
            .unwrap()
            .to_string_lossy()
    ));
    let mut info_file = File::create(info_path).unwrap();
    write!(&mut info_file, "[Samples]\r\n").unwrap();
    for wave_entry in &wave_entries {
        write!(
            &mut info_file,
            "\r\n    SampleName={}\r\n",
            name_to_str(&wave_entry.name),
        )
        .unwrap();
        write!(
            &mut info_file,
            "        SampleRate={}\r\n",
            wave_entry.sample_rate,
        )
        .unwrap();
        write!(
            &mut info_file,
            "        Key={}\r\n",
            semitone_tuning(wave_entry.original_pitch),
        )
        .unwrap();
        write!(
            &mut info_file,
            "        FineTune={}\r\n",
            cents_tuning(wave_entry.original_pitch)
        )
        .unwrap();
        write!(&mut info_file, "        Type=1\r\n").unwrap();
    }

    write!(&mut info_file, "\r\n\r\n[Instruments]\r\n").unwrap();
    for (i, program_entry) in program_entries.iter().enumerate() {
        write!(
            &mut info_file,
            "\r\n    InstrumentName={}\r\n",
            name_to_str(&program_entry.name)
        )
        .unwrap();
        for program_zone in &program_zones[i] {
            write!(
                &mut info_file,
                "\r\n        Sample={}\r\n",
                name_to_str(&wave_entries[program_zone.wave_index as usize].name)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_coarseTune={}\r\n",
                semitone_tuning(program_zone.pitch_finetuning)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_fineTune={}\r\n",
                cents_tuning(program_zone.pitch_finetuning)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_reverbEffectsSend={}\r\n",
                program_zone.reverb * 10
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_pan={}\r\n",
                pan_convert(program_zone.pan_position)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_keynumToVolEnvHold={}\r\n",
                program_zone.keynum_hold
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_keynumToVolEnvDecay={}\r\n",
                program_zone.keynum_decay
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_attackVolEnv={}\r\n",
                secs_to_timecent(program_zone.volume_env.attack)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_holdVolEnv={}\r\n",
                secs_to_timecent(program_zone.volume_env.hold)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_decayVolEnv={}\r\n",
                secs_to_timecent(program_zone.volume_env.decay)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_sustainVolEnv={}\r\n",
                ((100.0 - program_zone.volume_env.sustain) * 10.0) as i32
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_releaseVolEnv={}\r\n",
                secs_to_timecent(program_zone.volume_env.release)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_delayVolEnv={}\r\n",
                secs_to_timecent(program_zone.volume_env.delay)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_delayModEnv={}\r\n",
                secs_to_timecent(program_zone.modul_env.delay)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_initialAttenuation={}\r\n",
                (10.0 * program_zone.volume_env_atten) as i32,
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_delayVibLFO={}\r\n",
                secs_to_timecent(program_zone.vib_delay)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_freqVibLFO={}\r\n",
                secs_to_timecent(program_zone.vib_frequency / 8.176)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_vibLfoToPitch={}\r\n",
                program_zone.vib_to_pitch as i32
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_LowKey={}\r\n",
                program_zone.note_low
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_HighKey={}\r\n",
                program_zone.note_high
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_LowVelocity={}\r\n",
                program_zone.velocity_low
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_HighVelocity={}\r\n",
                program_zone.velocity_high
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_attackModEnv={}\r\n",
                secs_to_timecent(program_zone.modul_env.attack)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_holdModEnv={}\r\n",
                secs_to_timecent(program_zone.modul_env.hold)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_decayModEnv={}\r\n",
                secs_to_timecent(program_zone.modul_env.decay)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_sustainModEnv={}\r\n",
                (program_zone.modul_env.sustain * 10.0) as i32
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_releaseModEnv={}\r\n",
                secs_to_timecent(program_zone.modul_env.release)
            )
            .unwrap();
            write!(
                &mut info_file,
                "            Z_modEnvToPitch={}\r\n",
                program_zone.modul_env_to_pitch as i32
            )
            .unwrap();
            if program_zone.root_key != -1 {
                write!(
                    &mut info_file,
                    "            Z_overridingRootKey={}\r\n",
                    program_zone.root_key
                )
                .unwrap();
            }
            write!(
                &mut info_file,
                "            Z_sampleModes={}\r\n",
                wave_entries[program_zone.wave_index as usize].loop_info
            )
            .unwrap();
        }

        write!(&mut info_file, "\r\n        GlobalZone\r\n\r\n").unwrap();
    }

    write!(&mut info_file, "\r\n[Presets]").unwrap();
    for (i, preset_entry) in preset_entries.iter().enumerate() {
        write!(
            &mut info_file,
            "\r\n\r\n    PresetName={}\r\n",
            name_to_str(&preset_entry.name),
        )
        .unwrap();
        write!(
            &mut info_file,
            "        Bank={}\r\n",
            preset_entry.midi_bank_number,
        )
        .unwrap();
        write!(
            &mut info_file,
            "        Program={}\r\n\r\n",
            preset_entry.midi_preset_number,
        )
        .unwrap();

        for zone in &preset_zones[i] {
            write!(
                &mut info_file,
                "        Instrument={}\r\n",
                name_to_str(&program_entries[zone.program_index as usize].name),
            )
            .unwrap();
            write!(&mut info_file, "            L_LowKey={}\r\n", zone.note_low).unwrap();
            write!(
                &mut info_file,
                "            L_HighKey={}\r\n",
                zone.note_high,
            )
            .unwrap();
            write!(
                &mut info_file,
                "            L_LowVelocity={}\r\n",
                zone.velocity_low,
            )
            .unwrap();
            write!(
                &mut info_file,
                "            L_HighVelocity={}\r\n",
                zone.velocity_high,
            )
            .unwrap();
            write!(&mut info_file, "\r\n        GlobalLayer\r\n").unwrap();
        }
    }

    write!(&mut info_file, "\r\n\r\n[Info]\r\n").unwrap();
    write!(&mut info_file, "Version=2.1\r\n").unwrap();
    write!(&mut info_file, "Engine=EMU8000\r\n").unwrap();
    write!(
        &mut info_file,
        "Name={}\r\n",
        args.mus_path
            .with_extension("")
            .file_stem()
            .unwrap()
            .to_string_lossy()
    )
    .unwrap();
    write!(&mut info_file, "Editor=Demus\r\n").unwrap();

    print!(
        "{}",
        header.display(
            platform,
            program_zones.iter().fold(0, |acc, xs| acc + xs.len()),
            preset_zones.iter().fold(0, |acc, xs| acc + xs.len()),
        )
    );
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
    num_streams: Option<i32>,
    stream_bpm: Option<i32>,
    stream_info_pointer: Option<i32>,
    num_labels: i32,
    offset_to_labels_offsets_table: i32,
    num_waves: i32,
    num_programs: i32,
    num_presets: i32,
}

impl MusHeader {
    fn parse(bytes: &mut impl Iterator<Item = u8>) -> Self {
        let magic = be_bytes!(bytes);
        let header_size = le_bytes!(bytes);
        let version_number = le_bytes!(bytes);

        Self {
            magic,
            header_size,
            version_number,
            reverb_volume: le_bytes!(bytes),
            reverb_type: le_bytes!(bytes),
            reverb_multiply: le_bytes!(bytes),
            num_sequences: le_bytes!(bytes),
            num_streams: if version_number == HEADER_VERSION_120 {
                Some(le_bytes!(bytes))
            } else {
                None
            },
            stream_bpm: if version_number == HEADER_VERSION_120 {
                Some(le_bytes!(bytes))
            } else {
                None
            },
            stream_info_pointer: if version_number == HEADER_VERSION_120 {
                Some(le_bytes!(bytes))
            } else {
                None
            },
            num_labels: le_bytes!(bytes),
            offset_to_labels_offsets_table: le_bytes!(bytes),
            num_waves: le_bytes!(bytes),
            num_programs: le_bytes!(bytes),
            num_presets: le_bytes!(bytes),
        }
    }

    fn display(&self, platform: Platform, sequence_zones: usize, preset_zones: usize) -> String {
        format!(
            "MUS header\n\
            Header bytes: {} bytes\n\
            MUS version: {}\n\
            System: {platform:?}\n\
            Reverb volume: {}\n\
            Reverb type: {}\n\
            Sequences: {}\n\
            {}\
            {}\
            Labels: {}\n\
            Samples: {} ({})\n\
            Instruments: {}\n\
            Instrument Zones: {sequence_zones}\n\
            Presets: {}\n\
            Preset zones: {preset_zones}\n\
            ",
            self.header_size,
            match self.version_number {
                HEADER_VERSION_114 => "1.14",
                HEADER_VERSION_120 => "1.20",
                _ => "UNKNOWN",
            },
            self.reverb_volume,
            self.reverb_type,
            self.num_sequences,
            self.num_streams
                .map(|num| format!("Streams: {num}\n"))
                .unwrap_or_default(),
            self.stream_bpm
                .map(|bpm| format!("Streams BPM: {bpm}\n"))
                .unwrap_or_default(),
            self.num_labels,
            self.num_waves,
            match platform {
                Platform::PC => "PCM16LE",
                Platform::Console => "SONY_4BIT_ADPCM",
            },
            self.num_programs,
            self.num_presets,
        )
    }
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
