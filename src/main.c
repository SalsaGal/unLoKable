#include <stdio.h>
#include <stdlib.h>

#define BUFFER_SIZE 1024 * 1024

int main(int argc, char *argv[]) {
  if (argc != 2) {
    printf("Incorrect argument count\n");
    return 1;
  }

  FILE *file = fopen(argv[1], "rb");
  char *buffer = malloc(BUFFER_SIZE);

  size_t bytes = fread(buffer, sizeof(char), BUFFER_SIZE, file);

  printf("%s\n", buffer);
  printf("Bytes: 0x%zu\n", bytes);

  if (bytes == BUFFER_SIZE) {
    printf("WARNING: Buffer full, might not have been fully read.\n");
  }

  return 0;
}
