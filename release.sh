# Linux build
cargo build --release
mkdir out

cp target/release/adsheader out
cp target/release/adsloopfind out
cp target/release/adsunloop out
cp target/release/cds2seq out
cp target/release/demul out
cp target/release/demus out
cp target/release/desnd out
cp target/release/msqsplit out
cp target/release/seqrepeat out
cp target/release/sf2panlaw out
cp target/release/vabfine out
cp target/release/vabsmp out
cp target/release/vagheader out
cp target/release/vagsanitizer out
cp target/release/vagunloop out

zip -r linux.zip out
rm -rf out

# Windows build
RUSTFLAGS='-L /usr/x86_64-w64-mingw32/lib' cargo zigbuild --target x86_64-pc-windows-gnu --release
mkdir out

cp target/x86_64-pc-windows-gnu/release/*.exe out

zip -r windows.zip out
rm -rf out
