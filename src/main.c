#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "structures_new_engine.h"

#define BUFFER_SIZE 1024 * 1024

bool is_magic_number(char *file) {
  return strncmp(file, "DNSa", 4) == 0;
}

int main(int argc, char *argv[]) {
  if (argc != 3) {
    printf("Incorrect argument count\n");
    return 1;
  }

  FILE *snd_file = fopen(argv[1], "rb");
  char *snd_buffer = malloc(BUFFER_SIZE);
  size_t snd_size = fread(snd_buffer, sizeof(char), BUFFER_SIZE, snd_file);

  FILE *smp_file = fopen(argv[2], "rb");
  char *smp_buffer = malloc(BUFFER_SIZE);
  size_t smp_size = fread(smp_buffer, sizeof(char), BUFFER_SIZE, smp_file);

  printf("SND size: 0x%zx\n", snd_size);
  printf("SMP size: 0x%zx\n", smp_size);

  if (snd_size == BUFFER_SIZE || smp_size == BUFFER_SIZE) {
    printf("WARNING: Buffer full, might not have been fully read.\n");
  }

  if (!is_magic_number(snd_buffer)) {
    printf("ERROR: Not a valid file\n");
    return 1;
  }

  return 0;
}
