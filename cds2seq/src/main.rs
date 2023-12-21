use clap::Parser;
use dbg_hex::dbg_hex;
use either::Either;
use std::io::Write;
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Parser)]
#[command(version)]
struct Args {
    input: PathBuf,
    #[clap(long, short)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let mut file = File::open(&args.input).expect("file cannot be opened");
    let mut contents = vec![];
    file.read_to_end(&mut contents).expect("file not readable");

    let mut content_iter = contents.iter().copied();

    let header = Header::load(&mut content_iter);
    dbg_hex!(&header);
    assert_eq!(0x5145_5361, header.magic, "invalid magic number");

    let body = content_iter.collect::<Vec<_>>();
    let tokens = Token::parse(&body);
    dbg_hex!(&tokens);

    let lexemes = Lexeme::lex(tokens);
    dbg_hex!(&lexemes);
    for lexeme in &lexemes {
        lexeme.visualise(0);
    }

    let mut output = vec![];
    for lexeme in &lexemes {
        lexeme.write_lexeme(&mut output);
    }

    let mut i = 0;
    while i < output.len() {
        let mut chunk = output.iter().skip(i).take(4);
        // TODO this might crash
        if *chunk.next().unwrap() == 0xff
            && *chunk.next().unwrap() == 0x32
            && *chunk.next().unwrap() == 0x01
        {
            output.splice(i..i + 4, [0xff, 0x2f, 0x00]);
            i += 3;
        } else {
            i += 1;
        }
    }

    dictionary(&mut output, header.quarter_note_time);

    let mut output_file = File::create(args.output.unwrap_or_else(|| {
        args.input.with_file_name(format!(
            "{}.seq",
            args.input.file_stem().unwrap().to_string_lossy()
        ))
    }))
    .unwrap();
    output_file
        .write_all(
            &[0x70, 0x51, 0x45, 0x53, 0x00, 0x00, 0x00, 0x01]
                .into_iter()
                .chain(header.ppqn.to_le_bytes())
                .chain(header.quarter_note_time.to_le_bytes().into_iter().skip(1))
                .chain([0x04, 0x02])
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let output_end = output
        .windows(3)
        .enumerate()
        .find_map(|(i, c)| {
            if *c == [0xff, 0x2f, 0x00] {
                Some(i)
            } else {
                None
            }
        })
        .unwrap();
    output_file.write_all(&output[0..output_end + 3]).unwrap();
}

fn dictionary(file: &mut Vec<u8>, quarter_note_time: u32) {
    const MAGIC: u16 = 0x51ff;

    let loop_terminator_count = file.windows(3).filter(|x| *x == [0xff, 0x2f, 0x00]).count();
    let mut loop_terminator_index = 0;

    let mut i = 0;
    while i < file.len() {
        let message = [file.get(i), file.get(i + 1), file.get(i + 2)]
            .into_iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        if message.len() != 3 {
            break;
        }

        let length = if message[0] == 0xff {
            match [message[1], message[2]] {
                [0xf1, 0x04] => Some(7),
                [0x39..=0x3f, 0x03] => Some(6),
                [0x4c | 0x4d | 0x14 | 0x15 | 0x18 | 0x33..=0x36, 0x02] => Some(5),
                [0x00 | 0x0e | 0x01 | 0x1a | 0x1c | 0x02 | 0x2e | 0x06 | 0x07 | 0x10 | 0x24
                | 0x31, 0x01] => Some(4),
                [0x03 | 0x08 | 0x09 | 0x41..=0x43 | 0x49, 0x00] => match file.get(i + 3) {
                    Some(0xff) => None,
                    _ => Some(3),
                },
                [0x05, 0x03] => {
                    file.splice(i..i + 3, [0xff, 0x51]);
                    i += 3;
                    None
                }
                [0x2f, 0x00] => {
                    loop_terminator_index += 1;
                    if loop_terminator_count > 1 && loop_terminator_index < loop_terminator_count {
                        Some(3)
                    } else {
                        None
                    }
                }
                [0x44, 0x00] => {
                    if loop_terminator_count == 0 {
                        file[i + 1] = 0x2f;
                    }
                    i += 3;
                    None
                }
                _ => None,
            }
        } else {
            None
        };

        if let Some(length) = length {
            file.splice(
                i..i + length,
                MAGIC
                    .to_ne_bytes()
                    .iter()
                    .chain(quarter_note_time.to_ne_bytes().iter().skip(1))
                    .copied(),
            );
            i += length;
        } else {
            i += 1;
        }
    }
}

#[derive(Debug)]
struct Header {
    magic: u32,
    quarter_note_time: u32,
    ppqn: u16,
    #[allow(unused)]
    version: u16,
}

impl Header {
    fn load(bytes: &mut impl Iterator<Item = u8>) -> Self {
        Header {
            magic: u32::from_be_bytes([
                bytes.next().unwrap(),
                bytes.next().unwrap(),
                bytes.next().unwrap(),
                bytes.next().unwrap(),
            ]),
            quarter_note_time: u32::from_be_bytes([
                bytes.next().unwrap(),
                bytes.next().unwrap(),
                bytes.next().unwrap(),
                bytes.next().unwrap(),
            ]),
            ppqn: u16::from_be_bytes([bytes.next().unwrap(), bytes.next().unwrap()]),
            version: u16::from_be_bytes([bytes.next().unwrap(), bytes.next().unwrap()]),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Token<'a> {
    /// `FF2E01XX` with `XX` being loop count.
    LoopStart(u8),
    /// Data without any sentinel values.
    Data(&'a [u8]),
    /// `FF2F00`.
    LoopFinish,
}

impl Token<'_> {
    fn parse(bytes: &[u8]) -> Vec<Token> {
        let mut i = 0;
        let mut tokens = vec![];
        while i < bytes.len() {
            if bytes[i] == 0xff {
                if bytes[i + 1] == 0x2e && bytes[i + 2] == 0x01 {
                    tokens.push(Token::LoopStart(bytes[i + 3]));
                    i += 4;
                    continue;
                } else if bytes[i + 1] == 0x2f && bytes[i + 2] == 0 {
                    tokens.push(Token::LoopFinish);
                    i += 3;
                    continue;
                }
            }
            if let Some(Token::Data(data)) = tokens.last_mut() {
                *data = &bytes[i - data.len()..=i];
            } else {
                tokens.push(Token::Data(&bytes[i..=i]));
            }
            i += 1;
        }

        tokens
    }
}

#[derive(Debug)]
enum Lexeme {
    Loop(u8, Vec<Lexeme>),
    Data(Vec<u8>),
}

impl Lexeme {
    fn visualise(&self, depth: usize) {
        match self {
            Self::Loop(count, children) => {
                println!("{}Loop: {}x", "\t".repeat(depth), count);
                for child in children {
                    child.visualise(depth + 1);
                }
            }
            Self::Data(data) => println!(
                "{}Data: {:#04x} .. {:#04x}",
                "\t".repeat(depth),
                data[0],
                data.last().unwrap(),
            ),
        }
    }

    fn lex(tokens: Vec<Token>) -> Vec<Lexeme> {
        let mut lexemes: Vec<Either<Token, Lexeme>> =
            tokens.iter().copied().map(Either::Left).collect::<Vec<_>>();

        let mut i = 0;
        while i < lexemes.len() {
            match lexemes[i] {
                Either::Left(Token::LoopFinish) => {
                    let mut j = i - 1;
                    lexemes.remove(i);
                    let mut loop_body = vec![];
                    loop {
                        match lexemes[j] {
                            Either::Left(Token::LoopStart(count)) => {
                                loop_body.reverse();
                                lexemes[j] = Either::Right(Lexeme::Loop(
                                    count,
                                    std::mem::take(&mut loop_body),
                                ));
                                i = j;
                                break;
                            }
                            Either::Left(token) => panic!("unexpected token: {token:?}"),
                            Either::Right(_) => loop_body.push(lexemes.remove(j).unwrap_right()),
                        }
                        j -= 1;
                    }
                }
                Either::Left(Token::Data(data)) => {
                    lexemes[i] = Either::Right(Lexeme::Data(data.to_vec()));
                }
                Either::Left(Token::LoopStart(_)) | Either::Right(_) => {}
            }

            i += 1;
        }

        lexemes.into_iter().map(Either::unwrap_right).collect()
    }

    fn write_lexeme(&self, file: &mut Vec<u8>) {
        match self {
            Lexeme::Data(data) => file.write_all(data).unwrap(),
            Lexeme::Loop(count, lexemes) => {
                file.write_all(&[0xff, 0x2e, 0x01, 0x00]).unwrap();
                for _ in 0..(*count).max(1) {
                    for lexeme in lexemes {
                        lexeme.write_lexeme(file);
                    }
                    file.write_all(&[0xff, 0x2f, 0x00]).unwrap();
                }
            }
        }
    }
}