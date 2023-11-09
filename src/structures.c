#include "structures.h"
#include "strings.h"
#include <stdio.h>

int parse_int(char **file) {
	int toReturn = ((*file)[0] & 0xff) * 0x00000001
		+ ((*file)[1] & 0xff) * 0x00000100
		+ ((*file)[2] & 0xff) * 0x00010000
		+ ((*file)[3] & 0xff) * 0x01000000;
	*file += 4;
	return toReturn;
}

// NEEDS TO BE 0xEE8C
unsigned short int parse_word(char **file) {
	unsigned short int toReturn = ((*file)[0] & 0xff) * 0x0100 + ((*file)[1] & 0xff);
	*file += 2;
	return toReturn;
}

unsigned char parse_byte(char **file) {
	unsigned char toReturn = (*file)[0] & 0xff;
	*file += 1;
	return toReturn;
}

SndHeader parse_header(char **file) {
	SndHeader header;
	header.magicNumber = parse_int(file);
	if (header.magicNumber != 0x61534e44) {
		printf(ERROR_INVALID_HEADER);
		exit(1);
	}
	header.headerSize = parse_int(file);
	header.bankVersion = parse_int(file);
	header.numPrograms = parse_int(file);
	header.numZones = parse_int(file);
	header.numWaves = parse_int(file);
	header.numSequences = parse_int(file);
	header.numLabels = parse_int(file);
	header.reverbMode = parse_int(file);
	header.reverbDepth = parse_int(file);
	return header;
}

SndProgram parse_program(char **file) {
	SndProgram program;
	program.numZones = parse_word(file);
	program.firstTone = parse_word(file);
	program.volume = parse_byte(file);
	program.panPos = parse_byte(file);
	parse_word(file);
	return program;
}

SndZone parse_zone(char **file) {
	SndZone zone;
	zone.priority = parse_byte(file);
	zone.parentProgram = parse_byte(file);
	zone.volume = parse_byte(file);
	zone.panPos = parse_byte(file);
	zone.rootKey = parse_byte(file);
	zone.pitchFinetuning = parse_byte(file);
	zone.noteLow = parse_byte(file);
	zone.noteHigh = parse_byte(file);
	zone.mode = parse_byte(file);
	zone.maxPitchRange = parse_byte(file);
	zone.ADSR1 = parse_word(file);
	zone.ADSR2 = parse_word(file);
	zone.waveIndex = parse_word(file) + 0x0100;
	return zone;
}

SndFile parse_snd_file(char **file, int file_length) {
	char *end_of_file = *file + file_length;
	
  SndFile toReturn;
  toReturn.header = parse_header(file);

  toReturn.programs = (SndProgram *) calloc(toReturn.header.numPrograms, sizeof(SndProgram));
  for (int i = 0; i < toReturn.header.numPrograms; i++) {
    toReturn.programs[i] = parse_program(file);
  }

  toReturn.zones = (SndZone *) calloc(toReturn.header.numZones, sizeof(SndZone));
  for (int i = 0; i < toReturn.header.numZones; i++) {
    toReturn.zones[i] = parse_zone(file);
  }

  toReturn.waveOffsets = (int *) calloc(toReturn.header.numWaves, sizeof(int));
  for (int i = 0; i < toReturn.header.numWaves; i++) {
    toReturn.waveOffsets[i] = parse_int(file);
  }

  toReturn.sequenceOffsets = (int *) calloc(toReturn.header.numSequences, sizeof(int));
  for (int i = 0; i < toReturn.header.numSequences; i++) {
    toReturn.sequenceOffsets[i] = parse_int(file);
  }

  toReturn.labels = (int *) calloc(toReturn.header.numLabels, sizeof(int));
  for (int i = 0; i < toReturn.header.numLabels; i++) {
    toReturn.labels[i] = parse_int(file);
  }

	int sequenceDataSize = end_of_file - *file;
  toReturn.sequenceSlices = (Slice *) calloc(toReturn.header.numSequences, sizeof(Slice));
  for (int i = 0; i < toReturn.header.numSequences; i++) {
		toReturn.sequenceSlices[i].start = *file + toReturn.sequenceOffsets[i];
		if (i == toReturn.header.numSequences - 1) {
			toReturn.sequenceSlices[i].length = sequenceDataSize - toReturn.sequenceOffsets[i];
		} else {
			toReturn.sequenceSlices[i].length = toReturn.sequenceOffsets[i + 1] - toReturn.sequenceOffsets[i];
		}
  }

  return toReturn;
}

SmpFile parse_smp_file(char **file, SndFile *snd, int length) {
	char *end_of_file = *file + length;

	SmpFile toReturn;
	toReturn.magicNumber[0] = parse_byte(file);
	toReturn.magicNumber[1] = parse_byte(file);
	toReturn.magicNumber[2] = parse_byte(file);
	toReturn.magicNumber[3] = parse_byte(file);
	toReturn.bodySize = parse_int(file);

	int waveDataSize = end_of_file - *file;
	toReturn.waves = (Slice *) calloc(snd->header.numWaves, sizeof(Slice));
	for (int i = 0; i < snd->header.numWaves; i++) {
		toReturn.waves[i].start = *file + snd->waveOffsets[i];
		if (i == snd->header.numWaves - 1) {
			toReturn.waves[i].length = waveDataSize - snd->waveOffsets[i];
		} else {
			toReturn.waves[i].length = snd->waveOffsets[i + 1] - snd->waveOffsets[i];
		}
	}

	return toReturn;
}
