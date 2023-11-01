#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "structures.h"

#define BUFFER_SIZE 1024 * 1024

char *loadBuffer(char *path) {
  FILE *file = fopen(path, "rb");;
  char *buffer = malloc(BUFFER_SIZE);
  size_t size = fread(buffer, sizeof(char), BUFFER_SIZE, file);

  if (size == BUFFER_SIZE) {
    printf("WARNING: Buffer full, might not have been fully read.\n");
  }

  return buffer;
}

int main(int argc, char *argv[]) {
  if (argc != 3) {
    printf("Incorrect argument count\n");
    return 1;
  }

  char *snd_buffer = loadBuffer(argv[1]);
  char *smp_buffer = loadBuffer(argv[2]);

  SndFile snd = parseSndFile(&snd_buffer);

  return 0;
}
