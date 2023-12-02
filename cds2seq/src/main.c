#include "../../lib/misc.h"
#include "../../lib/structures.h"
#include <stdbool.h>
#include <stdio.h>

bool reached_loop_count(unsigned char *index) {
  return (index[0] == 0xff) && (index[1] == 0x2e) && (index[2] == 0x01);
}

bool reached_loop_terminator(unsigned char *index) {
  return (index[0] == 0xff) && (index[1] == 0x2f) && (index[2] == 0x00);
}

bool reached_terminator(unsigned char *index) {
  return (index[0] == 0xff) && (index[1] == 0x44) && (index[2] == 0x00);
}

void copy_bytes(Vec *vec, int length, unsigned char data[]) {
  for (int i = 0; i < length; i++) {
    vec_push(vec, data[i]);
  }
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

  Vec output = vec_new(64);
  unsigned char *copy_start = cds_index;
  short loop_count = -1;
  while (true) {
    if (reached_loop_count(cds_index)) {
      loop_count = (short) cds_index[3];
      cds_index += 4;
      copy_bytes(&output, cds_index - copy_start, copy_start);
      printf("Copied 0x%lx to 0x%lx\n", copy_start - cds_buffer.start, cds_index - cds_buffer.start);
      copy_start = cds_index;
    } else if (reached_loop_terminator(cds_index)) {
      cds_index += 3;
      if (loop_count <= 1) {
        copy_bytes(&output, cds_index - copy_start, copy_start);
      } else {
        for (int i = 0; i < loop_count; i++) {
          copy_bytes(&output, cds_index - copy_start, copy_start);
        }
      }
      printf("Copied 0x%lx to 0x%lx, loop count %d\n", copy_start - cds_buffer.start, cds_index - cds_buffer.start, loop_count);
      copy_start = cds_index;
      loop_count = -1;
    } else if (reached_terminator(cds_index)) {
      cds_index += 3;
      copy_bytes(&output, cds_index - copy_start, copy_start);
      break;
    } else {
      cds_index++;
    }
  }

  FILE *out = fopen("test.bin", "wb");
  for (int i = 0; i < output.length; i++) {
    fprintf(out, "%c", output.start[i]);
  }
  fclose(out);

  return 0;
}
