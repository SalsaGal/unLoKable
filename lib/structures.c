#include "structures.h"
#include "strings.h"
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

Slice slice_new(unsigned char *data, int length) {
  Slice slice;
  slice.start = data;
  slice.length = length;
  return slice;
}

Vec vec_new(int capacity) {
  Vec vec;
  vec.start = (unsigned char *)calloc(capacity, sizeof(char));
  vec.length = 0;
  vec.capacity = capacity;
  return vec;
}

void vec_push(Vec *vec, unsigned char data) {
  if (vec->length >= vec->capacity) {
    Vec new_vec = vec_new(vec->capacity * 2);
    memcpy(new_vec.start, vec->start, vec->length * sizeof(char));
    new_vec.length = vec->length;
    *vec = new_vec;
  }
  vec->start[vec->length] = data;
  vec->length++;
}

float parse_float_be(unsigned char **file) {
  int i = parse_int_be(file);
  return *(float *)&i;
}

int parse_int_le(unsigned char **file) {
  int toReturn =
      ((*file)[3] & 0xff) * 0x00000001 + ((*file)[2] & 0xff) * 0x00000100 +
      ((*file)[1] & 0xff) * 0x00010000 + ((*file)[0] & 0xff) * 0x01000000;
  *file += 4;
  return toReturn;
}

int parse_int_be(unsigned char **file) {
  int toReturn =
      ((*file)[0] & 0xff) * 0x00000001 + ((*file)[1] & 0xff) * 0x00000100 +
      ((*file)[2] & 0xff) * 0x00010000 + ((*file)[3] & 0xff) * 0x01000000;
  *file += 4;
  return toReturn;
}

unsigned short int parse_word_le(unsigned char **file) {
  unsigned short int toReturn =
      ((*file)[0] & 0xff) * 0x0100 + ((*file)[1] & 0xff);
  *file += 2;
  return toReturn;
}

unsigned short int parse_word_be(unsigned char **file) {
  unsigned short int toReturn =
      ((*file)[1] & 0xff) * 0x0100 + ((*file)[0] & 0xff);
  *file += 2;
  return toReturn;
}

unsigned char parse_byte(unsigned char **file) {
  unsigned char toReturn = (*file)[0] & 0xff;
  *file += 1;
  return toReturn;
}

SndHeader parse_snd_header(unsigned char **file) {
  SndHeader header;
  header.magicNumber = parse_int_be(file);
  if (header.magicNumber != 0x61534e44) {
    printf(ERROR_INVALID_HEADER);
    exit(1);
  }
  header.headerSize = parse_int_be(file);
  header.bankVersion = parse_int_be(file);
  header.numPrograms = parse_int_be(file);
  header.numZones = parse_int_be(file);
  header.numWaves = parse_int_be(file);
  header.numSequences = parse_int_be(file);
  header.numLabels = parse_int_be(file);
  header.reverbMode = parse_int_be(file);
  header.reverbDepth = parse_int_be(file);
  return header;
}

SndProgram parse_program(unsigned char **file) {
  SndProgram program;
  program.numZones = parse_word_le(file);
  program.firstTone = parse_word_le(file);
  program.volume = parse_byte(file);
  program.panPos = parse_byte(file);
  parse_word_le(file);
  return program;
}

SndZone parse_zone(unsigned char **file, bool cents_tuning) {
  SndZone zone;
  zone.priority = parse_byte(file);
  zone.parentProgram = parse_byte(file);
  zone.volume = parse_byte(file);
  zone.panPos = parse_byte(file);
  zone.rootKey = parse_byte(file);
  zone.pitchFinetuning = cents_tuning ? (char)(((float)parse_byte(file)) * 100.0 / 128.0) : parse_byte(file);
  zone.noteLow = parse_byte(file);
  zone.noteHigh = parse_byte(file);
  zone.mode = parse_byte(file);
  zone.maxPitchRange = parse_byte(file);
  zone.ADSR1 = parse_word_le(file);
  zone.ADSR2 = parse_word_le(file);
  zone.waveIndex = parse_word_le(file) + 0x0100;
  return zone;
}

SndFile parse_snd_file(unsigned char **file, int file_length, bool cents_tuning) {
  unsigned char *end_of_file = *file + file_length;

  SndFile toReturn;
  toReturn.header = parse_snd_header(file);

  toReturn.programs =
      (SndProgram *)calloc(toReturn.header.numPrograms, sizeof(SndProgram));
  if (toReturn.programs == NULL) {
    printf(ERROR_OOM);
    exit(EXIT_FAILURE);
  }
  for (int i = 0; i < toReturn.header.numPrograms; i++) {
    toReturn.programs[i] = parse_program(file);
  }

  toReturn.zones = (SndZone *)calloc(toReturn.header.numZones, sizeof(SndZone));
  if (toReturn.zones == NULL) {
    printf(ERROR_OOM);
    exit(EXIT_FAILURE);
  }
  for (int i = 0; i < toReturn.header.numZones; i++) {
    toReturn.zones[i] = parse_zone(file, cents_tuning);
  }

  toReturn.waveOffsets = (int *)calloc(toReturn.header.numWaves, sizeof(int));
  if (toReturn.waveOffsets == NULL) {
    printf(ERROR_OOM);
    exit(EXIT_FAILURE);
  }
  for (int i = 0; i < toReturn.header.numWaves; i++) {
    toReturn.waveOffsets[i] = parse_int_be(file);
  }

  toReturn.sequenceOffsets =
      (int *)calloc(toReturn.header.numSequences, sizeof(int));
  if (toReturn.sequenceOffsets == NULL) {
    printf(ERROR_OOM);
    exit(EXIT_FAILURE);
  }
  for (int i = 0; i < toReturn.header.numSequences; i++) {
    toReturn.sequenceOffsets[i] = parse_int_be(file);
  }

  toReturn.labels = (int *)calloc(toReturn.header.numLabels, sizeof(int));
  if (toReturn.labels == NULL) {
    printf(ERROR_OOM);
    exit(EXIT_FAILURE);
  }
  for (int i = 0; i < toReturn.header.numLabels; i++) {
    toReturn.labels[i] = parse_int_be(file);
  }

  int sequenceDataSize = end_of_file - *file;
  toReturn.sequenceSlices =
      (Slice *)calloc(toReturn.header.numSequences, sizeof(Slice));
  if (toReturn.sequenceSlices == NULL) {
    printf(ERROR_OOM);
    exit(EXIT_FAILURE);
  }
  for (int i = 0; i < toReturn.header.numSequences; i++) {
    toReturn.sequenceSlices[i].start = *file + toReturn.sequenceOffsets[i];
    if (i == toReturn.header.numSequences - 1) {
      toReturn.sequenceSlices[i].length =
          sequenceDataSize - toReturn.sequenceOffsets[i];
    } else {
      toReturn.sequenceSlices[i].length =
          toReturn.sequenceOffsets[i + 1] - toReturn.sequenceOffsets[i];
    }
  }

  return toReturn;
}

SmpFile parse_smp_file(unsigned char **file, SndFile *snd, int length) {
  unsigned char *end_of_file = *file + length;

  SmpFile toReturn;
  toReturn.magicNumber[0] = parse_byte(file);
  toReturn.magicNumber[1] = parse_byte(file);
  toReturn.magicNumber[2] = parse_byte(file);
  toReturn.magicNumber[3] = parse_byte(file);
  toReturn.bodySize = parse_int_be(file);

  int waveDataSize = end_of_file - *file;
  toReturn.waves = (Slice *)calloc(snd->header.numWaves, sizeof(Slice));
  if (toReturn.waves == NULL) {
    printf(ERROR_OOM);
    exit(EXIT_FAILURE);
  }
  for (int i = 0; i < snd->header.numWaves; i++) {
    toReturn.waves[i].start = *file + snd->waveOffsets[i];
    if (i == snd->header.numWaves - 1) {
      toReturn.waves[i].length = waveDataSize - snd->waveOffsets[i];
    } else {
      toReturn.waves[i].length = snd->waveOffsets[i + 1] - snd->waveOffsets[i];
    }
  }

  return toReturn;
}

MsqHeader parse_msq_header(unsigned char **file) {
  MsqHeader to_return;
  to_return.msqID = parse_int_be(file);
  to_return.quarterNoteTime = parse_int_be(file);
  to_return.ppqn = parse_word_be(file);
  to_return.version = parse_word_be(file);
  to_return.numTracks = parse_word_be(file);
  to_return.padding = parse_word_be(file);
  return to_return;
}

CdsHeader parse_cds_header(unsigned char **file) {
  CdsHeader to_return;
  to_return.magic = parse_int_le(file);
  to_return.version = parse_int_le(file);
  to_return.ppqn = parse_word_le(file);
  to_return.quarterNoteTime[0] = parse_byte(file);
  to_return.quarterNoteTime[1] = parse_byte(file);
  to_return.quarterNoteTime[2] = parse_byte(file);
  to_return.timeSignature = parse_word_le(file);
  return to_return;
}
