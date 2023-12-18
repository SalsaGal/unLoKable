use dbg_hex::dbg_hex;
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

    dbg_hex!(&header);

    let body = content_iter.collect::<Vec<_>>();
    let tokens = parse_file(&body);
    dbg_hex!(tokens);
}

#[derive(Debug)]
enum Tokens<'a> {
    /// `FF2E01XX` with `XX` being loop count.
    LoopStart(u8),
    /// Data without any sentinel values.
    Data(&'a [u8]),
    /// `FF2F00`.
    LoopFinish,
}

fn parse_file(bytes: &[u8]) -> Vec<Tokens> {
    let mut i = 0;
    let mut tokens = vec![];
    while i < bytes.len() {
        if bytes[i] == 0xff {
            if bytes[i + 1] == 0x2e && bytes[i + 2] == 0x01 {
                tokens.push(Tokens::LoopStart(bytes[i + 3]));
                i += 4;
                continue;
            } else if bytes[i + 1] == 0x2f && bytes[i + 2] == 0 {
                tokens.push(Tokens::LoopFinish);
                i += 3;
                continue;
            }
        }
        if let Some(Tokens::Data(data)) = tokens.last_mut() {
            *data = &bytes[i - data.len()..=i];
        } else {
            tokens.push(Tokens::Data(&bytes[i..i + 1]));
        }
        i += 1;
    }

    tokens
}
