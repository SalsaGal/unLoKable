#include "../../lib/strings.h"

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

  return EXIT_SUCCESS;
}
