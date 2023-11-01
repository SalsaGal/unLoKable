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
  int opt;
  while ((opt = getopt(argc, argv, "h")) != -1) {
    switch (opt) {
      case 'h':
        printf(HELP_MESSAGE);
        return 0;
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

  return 0;
}
