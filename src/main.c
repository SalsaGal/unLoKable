#include <assert.h>
#include <bits/getopt_core.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "strings.h"
#include "structures.h"

#define BUFFER_SIZE 1024 * 1024
#define HELP_MESSAGE "Usage: able [OPTIONS] [SND FILE] [SMP FILE]\n" \
                     "Rips audio from the Legacy of Kain\n" \
                     "  -h  Displays this help message\n" \
                     "  -v  Displays extra information about files being loaded\n"

char *loadBuffer(char *path) {
  FILE *file = fopen(path, "rb");;
  if (file == NULL) return NULL;
  char *buffer = malloc(BUFFER_SIZE);
  size_t size = fread(buffer, sizeof(char), BUFFER_SIZE, file);

  if (size == BUFFER_SIZE) {
    printf(WARNING_BUFFER_FULL);
  }

  return buffer;
}

int main(int argc, char *argv[]) {
  bool verbose = false;
  int opt;
  while ((opt = getopt(argc, argv, "hv")) != -1) {
    switch (opt) {
      case 'h':
        printf(HELP_MESSAGE);
        return 0;

      case 'v':
        verbose = true;
        break;
    }
  }

  if (optind >= argc - 1) {
    printf(ERROR_MISSING_ARGS);
    return 1;
  }

  char *snd_buffer = loadBuffer(argv[optind++]);
  char *smp_buffer = loadBuffer(argv[optind++]);

  if (snd_buffer == NULL) {
    printf(ERROR_INVALID_FILE, argv[optind - 2]);
    return 1;
  } else if (smp_buffer == NULL) {
    printf(ERROR_INVALID_FILE, argv[optind - 1]);
    return 1;
  }

  SndFile snd = parseSndFile(&snd_buffer);
  if (verbose) {
    printf("HEADER\n");
    printf("headerSize: %d\n", snd.header.headerSize);
    printf("bankVersion: %d\n", snd.header.bankVersion);
    printf("numPrograms: %d\n", snd.header.numPrograms);
    printf("numZones: %d\n", snd.header.numZones);
    printf("numWaves: %d\n", snd.header.numWaves);
    printf("numSequences: %d\n", snd.header.numSequences);
    printf("numLabels: %d\n", snd.header.numLabels);
    printf("reverbMode: %d\n", snd.header.reverbMode);
    printf("reverbDepth: %d\n", snd.header.reverbDepth);

    for (int i = 0; i < snd.header.numPrograms; i++) {
      printf("\nPROGRAM %d\n", i);
      printf("numZones: %d\n", snd.programs[i].numZones);
      printf("firstTone: %d\n", snd.programs[i].firstTone);
      printf("volume: %d\n", snd.programs[i].volume);
      printf("panPos: %d\n", snd.programs[i].panPos);
    }

    for (int i = 0; i < snd.header.numZones; i++) {
      printf("\nZONE %d\n", i);
      printf("priority: %d\n", snd.zones[i].priority);
      printf("parentProgram: %d\n", snd.zones[i].parentProgram);
      printf("volume: %d\n", snd.zones[i].volume);
      printf("panPos: %d\n", snd.zones[i].panPos);
      printf("rootKey: %d\n", snd.zones[i].rootKey);
      printf("pitchFinetuning: %d\n", snd.zones[i].pitchFinetuning);
      printf("noteLow: %d\n", snd.zones[i].noteLow);
      printf("noteHigh: %d\n", snd.zones[i].noteHigh);
      printf("mode: %d\n", snd.zones[i].mode);
      printf("maxPitchRange: %d\n", snd.zones[i].maxPitchRange);
      printf("ADSR1: %d\n", snd.zones[i].ADSR1);
      printf("ADSR2: %d\n", snd.zones[i].ADSR2);
      printf("waveIndex: %d\n", snd.zones[i].waveIndex);
    }

    printf("\n");
    for (int i = 0; i < snd.header.numWaves; i++) {
      printf("Wave Offset %d: %d\n", i, snd.waveOffsets[i]);
    }

    printf("\n");
    for (int i = 0; i < snd.header.numSequences; i++) {
      printf("Sequence Offset %d: %d\n", i, snd.sequenceOffsets[i]);
    }

    printf("\n");
    for (int i = 0; i < snd.header.numLabels; i++) {
      printf("Label %d: %d\n", i, snd.labels[i]);
    }
  }

  return 0;
}
