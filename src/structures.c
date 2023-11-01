#include "structures.h"

int parseInt(char *file) {
	int toReturn =
		((int) file[0] * 0x00000001)
				+ ((int) file[1] * 0x00000100)
				+ ((int) file[2] * 0x00010000)
				+ ((int) file[3] * 0x01000000);
	return toReturn;
}

SndHeader parseHeader(char *file) {
	SndHeader header;
	header.magicNumber = parseInt(file);
	header.headerSize = parseInt(file + 4);
	header.bankVersion = parseInt(file + 8);
	header.numPrograms = parseInt(file + 12);
	header.numZones = parseInt(file + 16);
	header.numWaves = parseInt(file + 20);
	header.numSequences = parseInt(file + 24);
	header.numLabels = parseInt(file + 28);
	header.reverbMode = parseInt(file + 32);
	header.reverbDepth = parseInt(file + 36);
	return header;
}
