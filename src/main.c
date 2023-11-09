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
#define HELP_MESSAGE \
  "Usage: able [OPTIONS] [SND FILE] [SMP FILE]\n" \
  "Rips audio from the Legacy of Kain\n" \
  "  -h  Displays this help message\n" \
  "  -o  Specifies the path for the output directory, eg `-o song`\n"\
  "  -v  Displays extra information about files being loaded\n"

Slice load_buffer(char *path) {
  Slice to_return;

  FILE *file = fopen(path, "rb");
  // if (file == NULL) return NULL;
  to_return.start = malloc(BUFFER_SIZE);
  to_return.length = fread(to_return.start, sizeof(char), BUFFER_SIZE, file);

  if (to_return.length == BUFFER_SIZE) {
    printf(WARNING_BUFFER_FULL);
  }

  return to_return;
}

char *remove_path(char *path) {
  char *to_return = malloc(strlen(path));
  strcpy(to_return, path);
  char *end_of_to_return = to_return + strlen(path);

  while (end_of_to_return >= to_return) {
    if (*end_of_to_return == '/' || *end_of_to_return == '\\') {
      return end_of_to_return + 1;
    }
    end_of_to_return--;
  }

  return path;
}

char *remove_extension(char *path) {
  char *to_return = malloc(strlen(path));
  strcpy(to_return, path);
  char *end_of_to_return = to_return + strlen(path);

  while (end_of_to_return >= to_return) {
    if (*end_of_to_return == '.') {
      *end_of_to_return = '\0';
      break;
    }
    end_of_to_return--;
  }

  return to_return;
}

void make_directory(char *path) {
#if defined(_WIN32)
  _mkdir(path);
#elif defined(__linux__)
  #include <sys/stat.h>
  #include <sys/types.h>
  
  struct stat st = {0};
  if (stat(path, &st) == -1) {
    mkdir(path, 0777);
  } else {
    printf("Folder %s already exists!\n", path);
  }
#endif
}

int main(int argc, char *argv[]) {
  bool verbose = false;
  char *output_dir = NULL;
  int opt;
  while ((opt = getopt(argc, argv, "hvo:")) != -1) {
    switch (opt) {
      case 'h':
        printf(HELP_MESSAGE);
        return 0;

      case 'v':
        verbose = true;
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

  char *snd_path = argv[optind++];
  char *smp_path = argv[optind++];

  Slice snd_buffer = load_buffer(snd_path);
  char *snd_buffer_start = snd_buffer.start;
  Slice smp_buffer = load_buffer(smp_path);

  if (snd_buffer.start == NULL) {
    printf(ERROR_INVALID_FILE, argv[optind - 2]);
    return 1;
  } else if (smp_buffer.start == NULL) {
    printf(ERROR_INVALID_FILE, argv[optind - 1]);
    return 1;
  }

  SndFile snd = parse_snd_file(&snd_buffer.start, snd_buffer.length);
  SmpFile smp = parse_smp_file(&smp_buffer.start, &snd, smp_buffer.length);

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

    printf("\n");
    for (int i = 0; i < snd.header.numSequences; i++) {
      printf("Sequence %d: starts at %lu, length is %d\n", i, snd.sequenceSlices[i].start - snd_buffer_start, snd.sequenceSlices[i].length);
    }
  }

  char *snd_path_stripped;
  if (output_dir) {
    snd_path_stripped = output_dir;
  } else {
    snd_path_stripped = remove_extension(snd_path);
  }
  make_directory(snd_path_stripped);

  for (int i = 0; i < snd.header.numSequences; i++) {
    char *output_path = malloc(128); // TODO Make this better
    sprintf(output_path, "%s/%s_%04d.msq", snd_path_stripped, remove_path(snd_path_stripped), i);

    FILE *output = fopen(output_path, "wb");
    for (int j = 0; j < snd.sequenceSlices[i].length; j++) {
      fprintf(output, "%c", snd.sequenceSlices[i].start[j]);
    }
  }
  for (int i = 0; i < snd.header.numWaves; i++) {
    char *output_path = malloc(128); // TODO Make this better
    sprintf(output_path, "%s/%s_%04d.vag", snd_path_stripped, remove_path(snd_path_stripped), i);

    FILE *output = fopen(output_path, "wb");
    int sample_length = smp.waves[i].length;
    char header[48] = {
      0x56, 0x41, 0x47, 0x70,     // Magic number
      0, 0, 0, 3,                 // Version number
      0, 0, 0, 0,               // Padding
      (sample_length & 0xff000000) >> 24,    // Size
      (sample_length & 0xff0000) >> 16,
      (sample_length & 0xff00) >> 8,
      sample_length & 0xff,
      0x00, 0x00, 0xAC, 0x44, // Sample rate
      0, 0, 0, 0,             // Padding
      0, 0, 0, 0,
      0, 0, 0, 0,
      0, 0, 0, 0,             // Name
      0, 0, 0, 0,
      0, 0, 0, 0,
      0, 0, 0, 0,
    };
    for (int j = 0; j < 48; j++) {
      fprintf(output, "%c", header[j]);
    }
    for (int j = 0; j < smp.waves[i].length; j++) {
      fprintf(output, "%c", smp.waves[i].start[j]);
    }
  }

  char *vpr_output_path = malloc(128); // TODO Make this better
  sprintf(vpr_output_path, "%s/%s.vpr", snd_path_stripped, remove_path(snd_path_stripped));
  FILE *vpr_output = fopen(vpr_output_path, "wb");
  for (int i = 0; i < snd.header.numPrograms; i++) {
    SndProgram *program = &snd.programs[i];
    char toWrite[] = {
      (char) (program->numZones >> 8),
      program->volume,
      0,
      0,
      program->panPos,
      0, 0, 0, 0,
      0, 0, 0, 0,
      0, 0, 0,
    };
    for (int j = 0; j < 16; j++) {
      fprintf(vpr_output, "%c", toWrite[j]);
    }
  }
  for (int i = 0; i < 16 * (128 - snd.header.numPrograms); i++) {
    fprintf(vpr_output, "%c", 0);
  }

  char *vzn_output_path = malloc(128); // TODO Make this better
  sprintf(vzn_output_path, "%s/%s.vzn", snd_path_stripped, remove_path(snd_path_stripped));
  FILE *vzn_output = fopen(vzn_output_path, "wb");
  int current_parent_program = 0;
  int current_parent_program_streak = 0;
  for (int i = 0; i < snd.header.numZones; i++) {
    SndZone *zone = &snd.zones[i];
    if (zone->parentProgram != current_parent_program) {
      for (int j = 0; j < 32 * (16 - current_parent_program_streak); j++) {
        fprintf(vzn_output, "%c", 0);
      }
      current_parent_program = zone->parentProgram;
      current_parent_program_streak = 0;
    }

    char toWrite[] = {
      zone->priority,
      zone->mode,
      zone->volume,
      zone->panPos,
      zone->rootKey,
      zone->pitchFinetuning,
      zone->noteLow,
      zone->noteHigh,
      0, 0, 0, 0,
      zone->maxPitchRange,
      zone->maxPitchRange,
      0, 0,
      (char) ((zone->ADSR1 & 0xff00) >> 8),
      (char) (zone->ADSR1 & 0xff),
      (char) ((zone->ADSR2 & 0xff00) >> 8),
      (char) (zone->ADSR2 & 0xff),
      zone->parentProgram, 0,
      (char) ((zone->waveIndex & 0xff00) >> 8),
      (char) (zone->waveIndex & 0xff),
      0, 0, 0, 0,
      0, 0, 0, 0
    };
    for (int j = 0; j < 32; j++) {
      fprintf(vzn_output, "%c", toWrite[j]);
    }
    current_parent_program_streak++;
  }
  for (int i = 0; i < 32 * (16 - current_parent_program_streak); i++) {
    fprintf(vzn_output, "%c", 0);
  }
  
  return 0;
}
