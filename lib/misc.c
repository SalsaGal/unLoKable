#include "misc.h"
#include "strings.h"
#include "structures.h"
#include <stdio.h>
#include <string.h>

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
