use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

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
    #[clap(short, long)]
    output: Option<PathBuf>,
    #[clap(long)]
    attenuate: bool,
    #[clap(long)]
    amplify: bool,
}

fn main() {
    let args = Args::parse();

    let function = if args.amplify {
        Function::Amplify
    } else {
        Function::Attenuate
    };
    let file = std::fs::read_to_string(&args.input).unwrap();
    let mut lines = file.lines().map(|s| s.to_owned()).collect::<Vec<_>>();

    let z_pans = lines.iter().enumerate().filter_map(|(i, x)| {
        x.trim()
            .strip_prefix("Z_pan=")
            .map(|value| (i, u16_to_i16(value.parse::<u16>().unwrap()) as f32 / 500.0))
    });
    let z_atten = lines.iter().enumerate().filter_map(|(i, x)| {
        x.trim()
            .strip_prefix("Z_initialAttenuation=")
            .map(|value| (i, value.parse::<u32>().unwrap() as f32 / 25.0))
    });

    let shifted = z_pans
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

    let mut output = File::create(args.output.unwrap_or_else(|| {
        format!(
            "{}_panlawupd.{}",
            args.input.with_extension("").to_string_lossy(),
            args.input.extension().unwrap().to_string_lossy(),
        )
        .into()
    }))
    .unwrap();
    write!(output, "{}", lines.join("\r\n")).unwrap();
}

fn u16_to_i16(x: u16) -> i16 {
    unsafe { std::mem::transmute(x) }
}
