#include <stdio.h>
#include "../../lib/misc.h"
#include "../../lib/structures.h"

int main(int argc, char *argv[]) {
  if (argc != 2) {
    printf("Missing argument for file!\n");
    return 1;
  }

  Slice cds_buffer = load_buffer(argv[1]);
  char *cds_index = cds_buffer.start;
  CdsHeader header = parse_cds_header(&cds_index);

  printf("%x\n", header.magic);
  printf("%x\n", header.version);
  printf("%x\n", header.ppqn);
  printf("%x\n", header.quarterNoteTime[0]);
  printf("%x\n", header.quarterNoteTime[1]);
  printf("%x\n", header.quarterNoteTime[2]);
  printf("%x\n", header.timeSignature);

  return 0;
}
