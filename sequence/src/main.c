#include "structures.h"
#include <stdio.h>
#include <stdlib.h>

#define BUFFER_SIZE 1024 * 1024

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
	char *file_start = file_buffer;
  int file_length = fread(file_buffer, sizeof(char), BUFFER_SIZE, file);

  MsqHeader header = parse_header(&file_buffer);
	if (header.msqID != 0x614d5351 && header.msqID != 0x61534551) {
    printf("Incorrect magic number!\n");
    return 2;
  }

	int *track_offsets = calloc(header.numTracks, sizeof(int));
	for (int i = 0; i < header.numTracks; i++) {
		track_offsets[i] = parse_int(&file_buffer);
	}

	Slice *track_slices = calloc(header.numTracks, sizeof(Slice));
	for (int i = 0; i < header.numTracks; i++) {
		track_slices[i].start = file_start + track_offsets[i];
		if (i == header.numTracks - 1) {
			track_slices[i].length = file_length - track_offsets[i];
		} else {
			track_slices[i].length = track_offsets[i + 1] - track_offsets[i];
		}
	}

  printf("msqID: %d\n", header.msqID);
  printf("quarterNoteTime: %d\n", header.quarterNoteTime);
  printf("ppqn: %d\n", header.ppqn);
  printf("version: %d\n", header.version);
  printf("numTracks: %d\n", header.numTracks);
  printf("padding: %d\n", header.padding);
	for (int i = 0; i < header.numTracks; i++) {
		printf("Track #%d: %x, %x\n", i, track_offsets[i], track_slices[i].length);
	}

  return 0;
}
