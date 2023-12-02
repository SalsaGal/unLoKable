#include "../../lib/misc.h"
#include "../../lib/strings.h"
#include "../../lib/structures.h"
#include <stdio.h>
#include <stdlib.h>

#define BUFFER_SIZE 1024 * 1024

int main(int argc, char *argv[]) {
  if (argc != 3) {
    printf("Missing argument for file!\n");
    return 1;
  }

  char *output_dir = argv[2];
  FILE *file = fopen(argv[1], "rb");
  unsigned char *file_buffer = malloc(BUFFER_SIZE);
  unsigned char *file_start = file_buffer;
  int file_length =
      fread(file_buffer, sizeof(unsigned char), BUFFER_SIZE, file);

  MsqHeader header = parse_msq_header(&file_buffer);
  if (header.msqID != 0x614d5351 && header.msqID != 0x61534551) {
    printf("Incorrect magic number!\n");
    return 2;
  }

  int *track_offsets = calloc(header.numTracks, sizeof(int));
  if (track_offsets == NULL) {
    printf(ERROR_OOM);
    return EXIT_FAILURE;
  }
  for (int i = 0; i < header.numTracks; i++) {
    track_offsets[i] = parse_int_be(&file_buffer);
  }

  Slice *track_slices = calloc(header.numTracks, sizeof(Slice));
  if (track_slices == NULL) {
    printf(ERROR_OOM);
    return EXIT_FAILURE;
  }
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

  for (int i = 0; i < header.numTracks; i++) {
    char *output_path = malloc(128); // TODO Make this better
    if (output_path == NULL) {
      printf(ERROR_OOM);
      return EXIT_FAILURE;
    }
    sprintf(output_path, "%s/%s_%04d.cds", output_dir, remove_path(argv[1]), i);
    clean_path(output_path);
    FILE *output = fopen(output_path, "wb");
    if (output == NULL) {
      printf(ERROR_INVALID_FILE_CREATE, output_path);
      return EXIT_FAILURE;
    }
    Slice *track = &track_slices[i];
    unsigned char header_bytes[] = {
        0x70,
        0x53,
        0x44,
        0x43,
        0x0,
        0x0,
        0x0,
        0x1,
        (header.ppqn & 0xff00) >> 8,
        header.ppqn & 0xff,
        (header.quarterNoteTime & 0x00ff0000) >> 16,
        (header.quarterNoteTime & 0x0000ff00) >> 8,
        (header.quarterNoteTime & 0x000000ff),
        0x4,
        0x2,
    };
    for (int j = 0; j < 15; j++) {
      fprintf(output, "%c", header_bytes[j]);
    }
    for (int j = 0; j < track->length; j++) {
      fprintf(output, "%c", track->start[j]);
    }
  }

  return 0;
}
