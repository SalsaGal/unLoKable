use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Args {
    snd_path: PathBuf,
    smp_path: PathBuf,
    #[clap(short, long)]
    dreamcast: bool,
    #[clap(short)]
    cent_tuning: bool,
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let snd_file = std::fs::read(args.snd_path).unwrap();
    let smp_file = std::fs::read(args.smp_path).unwrap();
}

struct SndHeader {
    magic_number: i32,
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

struct SndProgram {
    num_zones: u16,
    first_tone: u16,
    volume: u8,
    pan_pos: u8,
}

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

struct SndFile<'a> {
    header: SndHeader,
    programs: Vec<SndProgram>,
    zones: Vec<SndZone>,
    wave_offsets: Vec<usize>,
    sequence_offsets: Vec<usize>,
    labels: Vec<usize>,
    sequences: &'a [u8],
}
