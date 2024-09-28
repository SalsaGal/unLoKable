use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use core::clap::{self, Parser};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Function {
    Attenuate,
    Amplify,
}

impl Function {
    fn signal_power_shift(&self, pan: f32, atten: f32) -> f32 {
        match self {
            Self::Attenuate => atten + (pan.abs() * (10.0 * f32::log10(2.0))),
            Self::Amplify => atten - (pan.abs() * (10.0 * f32::log10(2.0))),
        }
    }
}

#[derive(Parser)]
#[clap(version)]
struct Args {
    input: PathBuf,
    /// DEFAULT
    #[clap(long)]
    attenuate: bool,
    #[clap(long)]
    amplify: bool,
}

fn main() {
    core::init();

    let args = Args::parse();

    let function = if args.amplify {
        Function::Amplify
    } else {
        Function::Attenuate
    };

    for file in core::get_files(&args.input).unwrap() {
        convert(&file, function);
    }
}

fn convert(path: &Path, function: Function) {
    let file = std::fs::read_to_string(path).unwrap();
    let mut lines = file
        .lines()
        .map(std::borrow::ToOwned::to_owned)
        .collect::<Vec<_>>();

    let z_pans = lines
        .iter()
        .enumerate()
        .filter_map(|(i, x)| {
            x.trim()
                .strip_prefix("Z_pan=")
                .map(|value| (i, z_pan(value)))
        })
        .collect::<Vec<_>>();
    let z_atten = lines.iter().enumerate().filter_map(|(i, x)| {
        x.trim()
            .strip_prefix("Z_initialAttenuation=")
            .map(|value| (i, value.parse::<u32>().unwrap() as f32 / 25.0))
    });

    let pair_count = z_pans.len();
    let changed_attenuations = z_pans.iter().filter(|(_, x)| *x != 0.0).count();
    println!("Pair count: {pair_count}");
    println!("Changed Attenuations: {changed_attenuations}");

    if changed_attenuations == 0 {
        eprintln!("No attenuations changed, aborting");
        return;
    }

    let shifted = z_pans
        .into_iter()
        .zip(z_atten)
        .map(|((_, pan), (line, atten))| {
            (
                line,
                (function.signal_power_shift(pan, atten) * 25.0) as u32,
            )
        })
        .collect::<Vec<_>>();

    for (line, value) in shifted {
        lines[line] = format!("            Z_initialAttenuation={value}");
    }

    let mut output = File::create(format!(
        "{}_{}.{}",
        path.with_extension("").to_string_lossy(),
        if function == Function::Amplify {
            "amplified"
        } else {
            "attenuated"
        },
        path.extension().unwrap().to_string_lossy(),
    ))
    .unwrap();
    write!(output, "{}", lines.join("\r\n")).unwrap();
}

fn z_pan(value: &str) -> f32 {
    let integer = value
        .parse::<u16>()
        .map(u16_to_i16)
        .or_else(|_| value.parse::<i16>())
        .unwrap();

    integer as f32 / 500.0
}

fn u16_to_i16(x: u16) -> i16 {
    unsafe { std::mem::transmute(x) }
}
