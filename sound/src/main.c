#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "../../lib/misc.h"
#include "../../lib/strings.h"
#include "../../lib/structures.h"

#define HELP_MESSAGE                                                           \
  "Usage: able [OPTIONS] [SND FILE] [SMP FILE]\n"                              \
  "Rips audio from the Legacy of Kain\n"                                       \
  "  -h  Displays this help message\n"                                         \
  "  -d  Output Dreamcast style headers\n"                                     \
  "  -c  Converts pitch fine tuning values to cents\n"                         \
  "  -o  Specifies the path for the output directory, eg `-o song`\n"

int main(int argc, char *argv[]) {
  char *output_dir = NULL;
  bool dreamcast_style = false;
  bool cents_tuning = false;
  int opt;
  while ((opt = getopt(argc, argv, "hdco:")) != -1) {
    switch (opt) {
    case 'h':
      printf(HELP_MESSAGE);
      return 0;

    case 'd':
      dreamcast_style = true;
      break;

    case 'c':
      cents_tuning = true;
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
  Slice smp_buffer = load_buffer(smp_path);

  if (snd_buffer.start == NULL) {
    printf(ERROR_INVALID_FILE, argv[optind - 2]);
    return 1;
  } else if (smp_buffer.start == NULL) {
    printf(ERROR_INVALID_FILE, argv[optind - 1]);
    return 1;
  }

  SndFile snd = parse_snd_file(&snd_buffer.start, snd_buffer.length, cents_tuning);
  SmpFile smp = parse_smp_file(&smp_buffer.start, &snd, smp_buffer.length);

  char *output_folder_path;
  if (output_dir) {
    output_folder_path = output_dir;
  } else {
    output_folder_path = remove_extension(snd_path);
  }
  make_directory(output_folder_path);
  char *samples_folder_path = malloc(128); // TODO fix this lemao
  if (samples_folder_path == NULL) {
    printf(ERROR_OOM);
    return EXIT_FAILURE;
  }
  sprintf(samples_folder_path, "%s/samples", output_folder_path);
  make_directory(samples_folder_path);
  char *sequences_folder_path = malloc(128); // TODO fix this lemao
  if (sequences_folder_path == NULL) {
    printf(ERROR_OOM);
    return EXIT_FAILURE;
  }
  sprintf(sequences_folder_path, "%s/sequences", output_folder_path);
  make_directory(sequences_folder_path);

  for (int i = 0; i < snd.header.numSequences; i++) {
    char *output_path = malloc(128); // TODO Make this better
    if (output_path == NULL) {
      printf(ERROR_OOM);
      return EXIT_FAILURE;
    }
    sprintf(output_path, "%s/%s_%04d.msq", sequences_folder_path,
            remove_path(output_folder_path), i);
    clean_path(output_path);

    FILE *output = fopen(output_path, "wb");
    if (output == NULL) {
      printf(ERROR_INVALID_FILE_CREATE, output_path);
      return EXIT_FAILURE;
    }
    free(output_path);
    for (int j = 0; j < snd.sequenceSlices[i].length; j++) {
      fprintf(output, "%c", snd.sequenceSlices[i].start[j]);
    }
    fclose(output);
  }
  for (int i = 0; i < snd.header.numWaves; i++) {
    char *output_path = malloc(128); // TODO Make this better
    if (output_path == NULL) {
      printf(ERROR_OOM);
      return EXIT_FAILURE;
    }
    sprintf(output_path, "%s/%s_%04d.%s", samples_folder_path,
            remove_path(output_folder_path), i, dreamcast_style ? "dcs" : "vag");
    clean_path(output_path);

    FILE *output = fopen(output_path, "wb");
    if (output == NULL) {
      printf(ERROR_INVALID_FILE_CREATE, output_path);
      return EXIT_FAILURE;
    };
    free(output_path);
    int sample_length = smp.waves[i].length;
    unsigned char header[] = {
        0x56,
        0x41,
        0x47,
        0x70, // Magic number
        0,
        0,
        0,
        3, // Version number
        0,
        0,
        0,
        0,                                  // Padding
        (sample_length & 0xff000000) >> 24, // Size
        (sample_length & 0xff0000) >> 16,
        (sample_length & 0xff00) >> 8,
        sample_length & 0xff,
        0x00,
        0x00,
        0xAC,
        0x44, // Sample rate
        0,
        0,
        0,
        0, // Padding
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0, // Name
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    };
    if (!dreamcast_style) {
      for (int j = 0; j < 48; j++) {
        fprintf(output, "%c", header[j]);
      }
    }
    for (int j = 0; j < smp.waves[i].length; j++) {
      fprintf(output, "%c", smp.waves[i].start[j]);
    }
    fclose(output);
  }

  char *vbi_output_path = malloc(128); // TODO Make this better
  if (vbi_output_path == NULL) {
    printf(ERROR_OOM);
    return EXIT_FAILURE;
  }
  sprintf(vbi_output_path, "%s/%s.vbi", output_folder_path,
          remove_path(output_folder_path));
  clean_path(vbi_output_path);
  FILE *vbi_output = fopen(vbi_output_path, "wb");
  if (vbi_output == NULL) {
    printf(ERROR_INVALID_FILE_CREATE, vbi_output_path);
    return EXIT_FAILURE;
  }
  free(vbi_output_path);
  for (int i = 0; i < snd.header.numPrograms; i++) {
    SndProgram *program = &snd.programs[i];
    unsigned char toWrite[] = {
        (unsigned char)(program->numZones >> 8),
        program->volume,
        0,
        0,
        program->panPos,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    };
    for (int j = 0; j < 16; j++) {
      fprintf(vbi_output, "%c", toWrite[j]);
    }
  }
  for (int i = 0; i < 16 * (128 - snd.header.numPrograms); i++) {
    fprintf(vbi_output, "%c", 0);
  }

  int current_parent_program = 0;
  int current_parent_program_streak = 0;
  for (int i = 0; i < snd.header.numZones; i++) {
    SndZone *zone = &snd.zones[i];
    if (zone->parentProgram != current_parent_program) {
      for (int j = 0; j < 32 * (16 - current_parent_program_streak); j++) {
        fprintf(vbi_output, "%c", 0);
      }
      current_parent_program = zone->parentProgram;
      current_parent_program_streak = 0;
    }

    unsigned char toWrite[] = {zone->priority,
                               zone->mode,
                               zone->volume,
                               zone->panPos,
                               zone->rootKey,
                               zone->pitchFinetuning,
                               zone->noteLow,
                               zone->noteHigh,
                               0,
                               0,
                               0,
                               0,
                               zone->maxPitchRange,
                               zone->maxPitchRange,
                               0,
                               0,
                               (unsigned char)((zone->ADSR1 & 0xff00) >> 8),
                               (unsigned char)(zone->ADSR1 & 0xff),
                               (unsigned char)((zone->ADSR2 & 0xff00) >> 8),
                               (unsigned char)(zone->ADSR2 & 0xff),
                               zone->parentProgram,
                               0,
                               (unsigned char)((zone->waveIndex & 0xff00) >> 8),
                               (unsigned char)(zone->waveIndex & 0xff),
                               0,
                               0,
                               0,
                               0,
                               0,
                               0,
                               0,
                               0};
    for (int j = 0; j < 32; j++) {
      fprintf(vbi_output, "%c", toWrite[j]);
    }
    current_parent_program_streak++;
  }
  for (int i = 0; i < 32 * (16 - current_parent_program_streak); i++) {
    fprintf(vbi_output, "%c", 0);
  }

  char *snd_info_output_path = malloc(128); // TODO Make this better
  if (snd_info_output_path == NULL) {
    printf(ERROR_OOM);
    return EXIT_FAILURE;
  }
  sprintf(snd_info_output_path, "%s/%s_info.txt", output_folder_path,
          remove_path(output_folder_path));
  clean_path(snd_info_output_path);
  FILE *snd_info_output = fopen(snd_info_output_path, "w");
  if (snd_info_output == NULL) {
    printf(ERROR_OOM);
    return EXIT_FAILURE;
  }
  fprintf(snd_info_output, "Number of programs: %d\nNumber of zones: %d\n", snd.header.numPrograms, snd.header.numZones);
  fclose(snd_info_output);

  return EXIT_SUCCESS;
}
