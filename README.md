# Able

A set of tools made in Rust designed to extract music data from several old and proprietary file formats from Crystal Dynamics, then convert them into modern formats. More information can be found on the [wiki](https://github.com/SalsaGal/able/wiki).

## Programs

### adsheader

This program takes a raw binary file (presumed to be a headerless audio stream) and adds a simplified Sony ADS header to it. There are many complex variants of the ADS header, but this program uses the simplest one. ADS supports two main codecs, which are PCM16_LE and SONY_4BIT_ADPCM (or VAG) but others may be supported as well.
It also supports multichannel streams with interleave.

#### Usage

`adsheader [input_file] [channels] [samplerate] [interleave] [format]`

Both the number of channels and the samplerate must be greater than 0.
If the number of channels is 1, then interleave must be set to 0.
Format numbers:
1 = `PCM16_LE`
16 = `SONY_4BIT_ADPCM` (VAG)

The output will be an .ads file with the same name as the input file, unless the argument `-o` is given.

### adsloopfind

This program takes an ADS file with a simplified header as an input and it outputs its loop markers only when the codec is `SONY_4BIT_ADPCM` (VAG). If stream contains no loop markers or the codec is different, the program will output nothing.

#### Usage

`adsloopfind [input_file]`

If there is a loop, the output will be a text echo showing the sample-based loop markers.

#### Example

Command-line input:
`adsloopfind synth.ads`

Standard out:
`3164 11788 synth.wav`
