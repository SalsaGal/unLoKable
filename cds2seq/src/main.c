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

void write_byte(FILE *file, unsigned char byte) {
  fprintf(file, "%c", byte);
}

void write_dummy(FILE *file, unsigned char note_times[3]) {
  write_byte(file, 0xff);
  write_byte(file, 0x51);
  write_byte(file, note_times[0]);
  write_byte(file, note_times[1]);
  write_byte(file, note_times[2]);
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
  int loop_terminator_count = 0;
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
        loop_terminator_count++;
      } else {
        for (int i = 0; i < loop_count; i++) {
          copy_bytes(&output, cds_index - copy_start, copy_start);
          loop_terminator_count++;
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

  char *output_path = malloc(128); // TODO Make this better
  sprintf(output_path, "%s.seq", remove_extension(argv[1]));
  FILE *out = fopen(output_path, "wb");

  // Write header
  write_byte(out, 0x70);
  write_byte(out, 0x51);
  write_byte(out, 0x45);
  write_byte(out, 0x53);
  for (unsigned char *i = cds_buffer.start + 4; i < cds_buffer.start + 15; i++) {
    write_byte(out, *i);
  }

  // Write body
  int current_loop_terminator = 0;
  for (unsigned char *c = output.start; c < output.start + output.length; c++) {
    if (c[0] == 0xff && (c[1] == 0x00 || c[1] == 0x01 || c[1] == 0x02 || c[1] == 0x06 || c[1] == 0x07 || c[1] == 0x0e || c[1] == 0x10 || c[1] == 0x1a || c[1] == 0x1c || c[1] == 0x24 || c[1] == 0x2e || c[1] == 0x31 || c[1] == 0x32) && c[2] == 0x01) {
      write_dummy(out, header.quarterNoteTime);
      c += 3;
    } else if (c[0] == 0xff && (c[1] == 0x14 || c[1] == 0x15 || c[1] == 0x18 || c[1] == 0x33 || c[1] == 0x34 || c[1] == 0x35 || c[1] == 0x36 || c[1] == 0x4c || c[1] == 0x4d) && c[2] == 0x02) {
      write_dummy(out, header.quarterNoteTime);
      c += 4;
    } else if (c[0] == 0xff && (c[1] >= 0x39 && c[1] <= 0x3f) && c[2] == 0x03) {
      write_dummy(out, header.quarterNoteTime);
      c += 5;
    } else if (c[0] == 0xff && c[1] == 0xf1 && c[2] == 0x04) {
      write_dummy(out, header.quarterNoteTime);
      c += 6;
    } else if (c[0] == 0xff && (c[1] == 0x03 || c[1] == 0x08 || c[1] == 0x09 || c[1] == 0x41 || c[1] == 0x42 || c[1] == 0x43 || c[1] == 0x49) && c[2] == 0x00 && c[3] != 0xff) {
      write_dummy(out, header.quarterNoteTime);
      c += 2;
    } else if (c[0] == 0xff && c[1] == 0x05 && c[2] == 0x03) {
      write_byte(out, 0xff);
      write_byte(out, 0x51);
      c += 2;
    } else if (c[0] == 0xff && c[1] == 0xf0) {
      unsigned char text_length = c[2];
      write_dummy(out, header.quarterNoteTime);
      c += 2 + text_length;
    } else if (c[0] == 0xff && c[1] == 0x2f && c[2] == 0x00 && loop_terminator_count >= 1) {
      current_loop_terminator++;
      if (current_loop_terminator == loop_terminator_count) {
        write_byte(out, 0xff);
        write_byte(out, 0x2f);
        write_byte(out, 0x00);
      } else {
        write_dummy(out, header.quarterNoteTime);
      }
      c += 2;
    } else if (c[0] == 0xff && c[1] == 0x44 && c[2] == 0x00 && loop_terminator_count == 0) {
      write_byte(out, 0xff);
      write_byte(out, 0x2f);
      write_byte(out, 0x00);
      c += 2;
    } else {
      write_byte(out, *c);
    }
  }
  fclose(out);

  return 0;
}
