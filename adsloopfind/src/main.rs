use std::path::PathBuf;

const MAGIC_NUMBER: [u8; 4] = [0x53, 0x53, 0x68, 0x64];

fn main() {
    let ads_path = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("`ads` path must be supplied"),
    );
    let ads_file = std::fs::read(&ads_path).unwrap();

    let (lb, le) = find_loops(&ads_file);
    print!(
        "{lb} {le} {}\r\n",
        ads_path
            .with_extension("wav")
            .file_name()
            .unwrap()
            .to_string_lossy()
    );
}

fn find_loops(ads_file: &[u8]) -> (u32, u32) {
    if ads_file[0..4] != MAGIC_NUMBER {
        eprintln!(
            "Invalid magic number, expected {MAGIC_NUMBER:?}, found {:?}",
            &ads_file[0..4]
        );
        std::process::exit(1);
    }

    let body_size = load_bytes(&ads_file[0x24..]);
    let codec = load_bytes(&ads_file[8..]);
    if codec != 0x10 {
        return (0, 0);
    }

    let channel_number = load_bytes(&ads_file[0x10..]);
    let step_size = 16;

    let body = &ads_file[0x28..];

    body.chunks(step_size as usize)
        .enumerate()
        .find_map(|(i, x)| {
            if x[1] == 6 {
                Some((
                    i as u32 * 28 / channel_number,
                    body_size / 16 * 28 / channel_number,
                ))
            } else {
                None
            }
        })
        .unwrap_or((0, 0))
}

fn load_bytes(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}
