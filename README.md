# unLoKable

A set of tools made in Rust designed to extract music data from several old and proprietary file formats from Crystal Dynamics, then convert them into modern formats. More information can be found on the [wiki](https://github.com/SalsaGal/unlokable/wiki).

## Programs

### adsheader

This program takes a raw binary file (presumed to be a headerless audio stream) and adds a simplified Sony [ADS](https://github.com/SalsaGal/unlokable/wiki/File-Format:-ADS) header to it. There are many complex variants of the ADS header, but this program uses the simplest one. ADS supports two main codecs, which are `PCM16_LE` and `SONY_4BIT_ADPCM` (or [VAG](https://github.com/SalsaGal/unlokable/wiki/File-Format:-VAG)) but others may be supported as well. It also supports multichannel streams with interleave.

#### Usage

`adsheader [input_file] [channels] [samplerate] [interleave] [format]`

Both the number of channels and the samplerate must be greater than 0.
If the number of channels is 1, then interleave must be set to 0.
Format numbers:
1 = `PCM16_LE`
16 = `SONY_4BIT_ADPCM` (VAG)

The output will be an .ads file with the same name as the input file, unless the argument `-o` is given.

### adsloopfind

This program takes an [ADS](https://github.com/SalsaGal/unlokable/wiki/File-Format:-ADS) file with a simplified header as an input and it outputs its loop markers only when the codec is `SONY_4BIT_ADPCM` ([VAG](https://github.com/SalsaGal/unlokable/wiki/File-Format:-VAG)). If stream contains no loop markers or the codec is different, the program will output nothing.

#### Usage

`adsloopfind [input_file]`

If there is a loop, the output will be a text echo showing the sample-based loop markers.

#### Example

Command-line input:
`adsloopfind synth.ads`

Standard out:
`3164 11788 synth.wav`

### cds2seq

This program takes a [CDS](https://github.com/SalsaGal/unlokable/wiki/File-Format:-CDS) file (a proprietary Crystal Dynamics sequence) as an input and converts it into a [SEQ](https://github.com/SalsaGal/unlokable/wiki/File-Format:-SEQ) file (Sony PlayStation sequence format). It also recursively unrolls all the nested loops that often occur in CDS files and makes sure the loop markers are balanced. Some information gets lost during the conversion process, mainly the custom meta commands that are still not well documented.

#### Usage

`cds2seq [input_file]`

During conversion, the program also displays some information about the CDS file.

### demul

This program extracts the raw contents of a [MUL](https://github.com/SalsaGal/unlokable/wiki/File-Format:-MUL) file, a format developed by Crystal Dynamics to store multiplexed data streams. The program mainly focuses on extracting the individual audio channels, but it outputs the remaining data as one binary file as well. The program always assumes the individual audio channels to be SONY_4BIT_ADPCM ([VAG](https://github.com/SalsaGal/unlokable/wiki/File-Format:-VAG)) streams, but other codecs may be used on different systems.

#### Usage

`demul [input_file]`

Output audio rate text file layout:
`input_file_audio_chX.bin channels samplerate interleave codec`

Channels is always set to 1.
Interleave is always set to 0.
Codec is always set to 16 (`SONY_4BIT_ADPCM`).
