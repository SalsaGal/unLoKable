#include "../../lib/misc.h"
#include "../../lib/structures.h"
#include "../../lib/strings.h"
#include "structures.h"

#include <getopt.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char *argv[]) {
  char *output_dir = NULL;
  bool pc_style = false;
  int opt;
  while ((opt = getopt(argc, argv, "hpo:")) != -1) {
    switch (opt) {
    case 'h':
      printf("TODO Help");
      return 0;

    case 'p':
      pc_style = true;
      break;

    case 'o':
      output_dir = optarg;
      break;
    }
  }

  if (optind >= argc - 1) {
    printf(ERROR_MISSING_ARGS);
    return 1;
  }

  char *mus_path = argv[optind++];
  char *sam_path = argv[optind++];

  Slice mus_buffer = load_buffer(mus_path);
  Slice sam_buffer = load_buffer(sam_path);

  unsigned char *mus_buffer_cursor = mus_buffer.start;

  MusHeader header = parse_mus_header(&mus_buffer_cursor);
  if (header.ID != 0x4D757321) {
    printf("Incorrect magic number!\n");
    return EXIT_FAILURE;
  }
	printf("ID: %d\n", header.ID);
	printf("headerSize: %d\n", header.headerSize);
	printf("versionNumber: %d\n", header.versionNumber);
	printf("reverbVolume: %d\n", header.reverbVolume);
	printf("reverbType: %d\n", header.reverbType);
	printf("reverbMultiply: %d\n", header.reverbMultiply);
	printf("numSequences: %d\n", header.numSequences);
	printf("numLabels: %d\n", header.numLabels);
	printf("offsetToLabelsOffsetsTable: %d\n", header.offsetToLabelsOffsetsTable);
	printf("numWaves: %d\n", header.numWaves);
	printf("numPrograms: %d\n", header.numPrograms);
	printf("numPresets: %d\n", header.numPresets);
  MsqTable *msq_tables = calloc(header.numSequences, sizeof(MsqTable));
  for (int i = 0; i < header.numSequences; i++) {
    msq_tables[i] = parse_msq_table(&mus_buffer_cursor);
    printf("MSQ table #%d: %x, %x\n", i, msq_tables[i].msqIndex, msq_tables[i].msqOffset);
  }
  int *layers = calloc(header.numPresets + header.numPrograms, sizeof(int));
  for (int i = 0; i < header.numPresets + header.numPrograms; i++) {
    layers[i] = parse_int_be(&mus_buffer_cursor);
    printf("Layer #%d: %x\n", i, layers[i]);
  }

  return EXIT_SUCCESS;
}
