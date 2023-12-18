use dbg_hex::dbg_hex;
use either::Either;
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Debug)]
struct Header {
    magic: u32,
    quarter_note_time: u32,
    ppqn: u16,
    version: u16,
}

fn main() {
    let path = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("argument needs to be supplied"),
    );
    let mut file = File::open(path).expect("file cannot be opened");
    let mut contents = vec![];
    file.read_to_end(&mut contents).expect("file not readable");

    let mut content_iter = contents.iter().copied();

    let header = Header {
        magic: u32::from_be_bytes([
            content_iter.next().unwrap(),
            content_iter.next().unwrap(),
            content_iter.next().unwrap(),
            content_iter.next().unwrap(),
        ]),
        quarter_note_time: u32::from_be_bytes([
            content_iter.next().unwrap(),
            content_iter.next().unwrap(),
            content_iter.next().unwrap(),
            content_iter.next().unwrap(),
        ]),
        ppqn: u16::from_be_bytes([content_iter.next().unwrap(), content_iter.next().unwrap()]),
        version: u16::from_be_bytes([content_iter.next().unwrap(), content_iter.next().unwrap()]),
    };
    assert_eq!(0x51455361, header.magic, "invalid magic number");

    // dbg_hex!(&header);

    let body = content_iter.collect::<Vec<_>>();
    let tokens = parse_file(&body);
    // dbg_hex!(&tokens);

    let lexemes = lex_file(tokens);
    dbg_hex!(&lexemes);
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

fn parse_file(bytes: &[u8]) -> Vec<Token> {
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
            tokens.push(Token::Data(&bytes[i..i + 1]));
        }
        i += 1;
    }

    tokens
}

#[derive(Debug)]
enum Lexeme {
    Loop(u8, Vec<Lexeme>),
    Data(Vec<u8>),
}

fn lex_file(tokens: Vec<Token>) -> Vec<Lexeme> {
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
                            lexemes[j] =
                                Either::Right(Lexeme::Loop(count, std::mem::take(&mut loop_body)));
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
                lexemes[i] = Either::Right(Lexeme::Data(data.to_vec()))
            }
            Either::Left(Token::LoopStart(_)) | Either::Right(_) => {}
        }

        i += 1;
    }

    lexemes.into_iter().map(Either::unwrap_right).collect()
}
