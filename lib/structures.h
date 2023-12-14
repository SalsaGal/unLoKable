#pragma once

#include <stdbool.h>
#include <stdlib.h>

typedef struct {
  unsigned char *start;
  int length;
} Slice;

// TODO Use this more
Slice slice_new(unsigned char *data, int length);

typedef struct {
  unsigned char *start;
  int length;
  int capacity;
} Vec;

Vec vec_new(int capacity);
void vec_push(Vec *vec, unsigned char data);

float parse_float_be(unsigned char **file);
int parse_int_le(unsigned char **file);
int parse_int_be(unsigned char **file);
unsigned short int parse_word_le(unsigned char **file);
unsigned short int parse_word_be(unsigned char **file);
unsigned char parse_byte(unsigned char **file);

typedef struct {
  int magicNumber;
  int headerSize;
  int bankVersion;
  int numPrograms;
  int numZones;
  int numWaves;
  int numSequences;
  int numLabels;
  int reverbMode;
  int reverbDepth;
} SndHeader;

SndHeader parse_snd_header(unsigned char **file);

typedef struct {
  unsigned short int numZones;
  unsigned short int firstTone;
  unsigned char volume;
  unsigned char panPos;
} SndProgram;

SndProgram parse_program(unsigned char **file);

typedef struct {
  unsigned char priority;
  unsigned char parentProgram;
  unsigned char volume;
  unsigned char panPos;
  unsigned char rootKey;
  unsigned char pitchFinetuning;
  unsigned char noteLow;
  unsigned char noteHigh;
  unsigned char mode;
  unsigned char maxPitchRange;
  unsigned short int ADSR1;
  unsigned short int ADSR2;
  unsigned short int waveIndex;
} SndZone;

SndZone parse_zone(unsigned char **file, bool cents_tuning);

typedef struct {
  SndHeader header;
  SndProgram *programs;
  SndZone *zones;
  int *waveOffsets;
  int *sequenceOffsets;
  int *labels;
  Slice *sequenceSlices;
} SndFile;

SndFile parse_snd_file(unsigned char **file, int file_length, bool cents_tuning);

typedef struct {
  unsigned char magicNumber[4]; // PMSa
  int bodySize;
  Slice *waves;
} SmpFile;

SmpFile parse_smp_file(unsigned char **file, SndFile *snd, int length);

typedef struct {
  int msqID;
  unsigned int quarterNoteTime;
  unsigned short int ppqn;
  unsigned short int version;
  unsigned short int numTracks;
  unsigned short int padding;
} MsqHeader;

MsqHeader parse_msq_header(unsigned char **file);

typedef struct {
  int magic;
  int version;
  unsigned short int ppqn;
  unsigned char quarterNoteTime[3];
  unsigned short int timeSignature;
} CdsHeader;

CdsHeader parse_cds_header(unsigned char **file);
