#include "structures.h"
#include <stdio.h>
#include <stdlib.h>

#define BUFFER_SIZE 1024 * 1024

int parse_int(char **file) {
	int toReturn = ((*file)[0] & 0xff) * 0x00000001
		+ ((*file)[1] & 0xff) * 0x00000100
		+ ((*file)[2] & 0xff) * 0x00010000
		+ ((*file)[3] & 0xff) * 0x01000000;
	*file += 4;
	return toReturn;
}

unsigned short int parse_word(char **file) {
	unsigned short int toReturn = ((*file)[1] & 0xff) * 0x0100 + ((*file)[0] & 0xff);
	*file += 2;
	return toReturn;
}

unsigned char parse_byte(char **file) {
	unsigned char toReturn = (*file)[0] & 0xff;
	*file += 1;
	return toReturn;
}

MsqHeader parse_header(char **file) {
  MsqHeader to_return;
  to_return.msqID = parse_int(file);
  to_return.quarterNoteTime = parse_int(file);
  to_return.ppqn = parse_word(file);
  to_return.version = parse_word(file);
  to_return.numTracks = parse_word(file);
  to_return.padding = parse_word(file);
  return to_return;
}

int main(int argc, char *argv[]) {
  if (argc != 2) {
    printf("Missing argument for file!\n");
    return 1;
  }

  FILE *file = fopen(argv[1], "rb");
  char *file_buffer = malloc(BUFFER_SIZE);
  fread(file_buffer, sizeof(char), BUFFER_SIZE, file);

  MsqHeader header = parse_header(&file_buffer);
	if (header.msqID != 0x614d5351 && header.msqID != 0x61534551) {
    printf("Incorrect magic number!\n");
    return 2;
  }

  printf("msqID: %d\n", header.msqID);
  printf("quarterNoteTime: %d\n", header.quarterNoteTime);
  printf("ppqn: %d\n", header.ppqn);
  printf("version: %d\n", header.version);
  printf("numTracks: %d\n", header.numTracks);
  printf("padding: %d\n", header.padding);

  return 0;
}
