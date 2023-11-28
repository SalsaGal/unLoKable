#include <stdbool.h>
#include <stdio.h>
#include "../../lib/misc.h"
#include "../../lib/structures.h"

bool reached_terminator(unsigned char *index) {
  return (index[0] == 0xff);// && (index[1] == 0x44) && (index[2] == 0x00);
}

int main(int argc, char *argv[]) {
  if (argc != 2) {
    printf("Missing argument for file!\n");
    return 1;
  }

  Slice cds_buffer = load_buffer(argv[1]);
  unsigned char *cds_index = cds_buffer.start;
  CdsHeader header = parse_cds_header(&cds_index);

  printf("%x\n", header.magic);
  printf("%x\n", header.version);
  printf("%x\n", header.ppqn);
  printf("%x\n", header.quarterNoteTime[0]);
  printf("%x\n", header.quarterNoteTime[1]);
  printf("%x\n", header.quarterNoteTime[2]);
  printf("%x\n", header.timeSignature);

  while (!reached_terminator(cds_index)) {
    printf("%x\n", ((int) *cds_index) % 0xff);
    cds_index++;
  }

  return 0;
}
