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
}
