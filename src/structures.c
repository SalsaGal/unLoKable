#include "structures.h"

int parseInt(char **file) {
	int toReturn =
		((int) (*file)[0] * 0x00000001)
				+ ((int) (*file)[1] * 0x00000100)
				+ ((int) (*file)[2] * 0x00010000)
				+ ((int) (*file)[3] * 0x01000000);
	*file += 4;
	return toReturn;
}

int parseWord(char **file) {
	int toReturn =
		((int) (*file)[0] * 0x0001)
				+ ((int) (*file)[1] * 0x0100);
	*file += 2;
	return toReturn;
}

int parseByte(char **file) {
	int toReturn = (int) (*file)[0];
	*file += 1;
	return toReturn;
}

SndHeader parseHeader(char **file) {
	SndHeader header;
	header.magicNumber = parseInt(file);
	header.headerSize = parseInt(file);
	header.bankVersion = parseInt(file);
	header.numPrograms = parseInt(file);
	header.numZones = parseInt(file);
	header.numWaves = parseInt(file);
	header.numSequences = parseInt(file);
	header.numLabels = parseInt(file);
	header.reverbMode = parseInt(file);
	header.reverbDepth = parseInt(file);
	return header;
}

SndProgram parseProgram(char **file) {
	SndProgram program;
	program.numZones = parseWord(file);
	program.firstTone = parseWord(file);
	program.volume = parseByte(file);
	program.panPos = parseByte(file);
	parseWord(file);
	return program;
}
