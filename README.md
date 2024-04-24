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

### demus

This program takes a MUS and a SAM file and decompiles their contents.

Most of the information from the MUS file gets converted to simple ASCII text and is saved into a txt file with the same name. The resulting text file follows a data layout specifically tuned to be used with SF2Comp (sf2comp.exe), a SoundFont compiler command-line utility for Windows. Note that in order for the utility to work, you need to retrieve `sfedt32.dll` separately and paste it into the same folder where the utility is located, it is not bundled directly with SF2Comp due to copyright. The help.txt file should be bundled with the utility inside the 'sf2cmp10.zip' file. Consult that for compile and decompile commands.

The samples are exported as ADS files and the sequences as MSQ files. The samples loop information gets exported onto a text file (*_smploopinfo.txt) that is formatted to be used with [LoopingAudioConverter](https://github.com/libertyernie/LoopingAudioConverter). If the samples come from a PlayStation 2 build of the game, the sample loop info text file needs to be re-built using the adsloopfind utility on each ads file. Preferably doable using a batch script, like 'adsloopfind_folder.bat'. Otherwise, the sample loop info text file can be used directly.

The ADS files need to be converted to WAV first, using tools such as [VGSC 2.0](https://wiki.vg-resource.com/Video_Game_Sound_Converter), [vgmstream](https://vgmstream.org/), [foobar2000](https://www.foobar2000.org/) with the vgmstream plugin and many others...
Once converted, LoopingAudioConverter is able to append the loop information to them as 'RIFF smpl' chunks. To use the sample loop info text file with LoopingAudioConverter, place it into the same folder, rename the file to 'loop.txt' and it should be automatically loaded once you launch the program. The remaining instructions should be on the 'About.html' file.

#### Usage

```
demus [mus_file] [sam_file]

Options:

-p, --pc (Tells the program to use the PC format. This is the default.)
-c, --console (Tells the program to use the console format.)
-o, --output (Output folder of the files. Defaults to the input with a different extension.)
```

The only difference between the PC version and the console version is the sample codec. The program currently supports PC and PlayStation 2 versions of the samples, but more codecs may be added in the future. The PC version uses PCM16_LE formatted samples while the PlayStation 2 version uses SONY_4BIT_ADPCM (VAG).desnd

### desnd

This program takes an SND and an SMP file and decompiles their contents.

Most of the information is then transformed and saved onto multiple files. The instrument and the sample information gets respectively saved onto a VH and a VB file. The samples are also separately saved as either VAG or DCS files. Then the sequences are saved as either CDS or MSQ files.

#### Usage

```
desnd [snd_file] [smp_file]

Options:

-f file_version (What version of the 'snd' file is being opened. Possible values: soul-reaver, prototype, gex. The default is soul-reaver.)
-d, --dreamcast (Tells the program that the files come from a Dreamcast game build.)
-o, --output (Output folder of the files. Defaults to the input with a different extension.)
```

By default the program supports files that come from PlayStation builds of the game. The Dreamcast builds may use a variety of codecs for the samples that the program currently does not handle. At the moment the Dreamcast samples get stored as headerless DCS files (VH and VB files may not be usable if the Dreamcast option is specified).

To convert a pair of VH and VB files into a single VAB file, simply concatenate their binary contents. Then to convert a VAB file into a modern format, such as SoundFont (.sf2) or DownLoadable Sounds (.dls), you can use [VGMTrans](https://github.com/vgmtrans/vgmtrans), [Awave Studio](https://www.fmjsoft.com/awavestudio.html#main) or possibly other utilities. Note that many pieces of information that will be saved onto those files will be incorrect and will need lots of laborious manual adjustment.

Things that get typically screwed up and need to be manually fixed:
- ADSR curves;
- Sample finetuning;
- Instrument tones/zones finetuning;
- Pan laws (it should be linear rather than based on -3.01dB);

Sometimes samples can also be subjected to clipping or overflow errors and may need to be re-exported/re-converted to WAV either from the source VAB file or the ripped VAG files.

Currently there's no easy fix for all the problems listed above and it requires time, patience and knowledge on how VAB files work, how samples work, how SoundFonts work and how to fix them. This is mostly for dedicated users. [Polyphone](https://www.polyphone-soundfonts.com/) is often recommended as an editor for fixing issues with broken SoundFont files.

### msqsplit

This program takes an MSQ file and splits into multiple CDS files.

#### Usage

```
msqsplit [msq_file]
```

CDS files can be later converted to SEQ using cds2seq and then to MIDI using loveemu's [seq2mid](https://github.com/loveemu/seq2mid). Before converting the SEQ files to MIDI, you may want to check whether they need to be looped or not using seqrepeat (preferably with the marker option enabled by default). If you want to have all the MIDI tracks from an MSQ file to be re-grouped/re-merged, you first need to convert all the MIDI files from Type 0 (SMF0) to Type 1 (SMF1). Then you can use VirtuosicAI's [MIDI Merger Lite](https://github.com/VirtuosicAI/MIDI-Merger-Lite) to merge them into a single MIDI file while having the option 'Skip the 1st track of non-primary MIDIs' enabled.

### seqrepeat

This program takes a Sony PlayStation sequence file (SEQ) and extends its runtime by repeating a marked section or the entire file.

#### Usage

```
seqrepeat [seq_file] [reading_count]

Options:

-m Whether to read from the tempo marker rather than the entire file
```

If a reading count of 1 is specified, the resulting output file will be exactly the same as the input one. This program does not currently seek standard loop markers for SEQ files. Instead it uses the first tempo change command as a reference loop start marker. Support for standard loop markers may be added in the future with a different argument/option.
