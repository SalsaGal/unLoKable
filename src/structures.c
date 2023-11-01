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
	return header;
}
