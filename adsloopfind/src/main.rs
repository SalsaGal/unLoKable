const MAGIC_NUMBER: [u8; 4] = [0x53, 0x53, 0x68, 0x64];

fn main() {
    let ads_path = std::env::args()
        .next()
        .expect("`ads` path must be supplied");
    let ads_file = std::fs::read(ads_path).unwrap();

    if ads_file[..4] != MAGIC_NUMBER {
        eprintln!(
            "Invalid magic number, expected {MAGIC_NUMBER:?}, found {:?}",
            &ads_file[..4]
        );
    }
}
