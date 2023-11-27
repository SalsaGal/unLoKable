#pragma once

#include <stdlib.h>

typedef struct {
  char *start;
  int length;
} Slice;

// TODO Use this more
Slice slice_new(char *data, int length);

typedef struct {
  char *start;
  int length;
  int capacity;
} Vec;

Vec vec_new(int capacity);
void vec_push(Vec *vec, char data);

int parse_int_le(char **file);
int parse_int_be(char **file);
unsigned short int parse_word_le(char **file);
unsigned short int parse_word_be(char **file);
unsigned char parse_byte(char **file);

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

SndHeader parse_snd_header(char **file);

typedef struct {
  unsigned short int numZones;
  unsigned short int firstTone;
  unsigned char volume;
  unsigned char panPos;
} SndProgram;

SndProgram parse_program(char **file);

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

SndZone parse_zone(char **file);

typedef struct {
  SndHeader header;
  SndProgram *programs;
  SndZone *zones;
  int *waveOffsets;
  int *sequenceOffsets;
  int *labels;
  Slice *sequenceSlices;
} SndFile;

SndFile parse_snd_file(char **file, int length);

typedef struct {
  char magicNumber[4]; // PMSa
  int bodySize;
  Slice *waves;
} SmpFile;

SmpFile parse_smp_file(char **file, SndFile *snd, int length);

typedef struct {
  int msqID;
  unsigned int quarterNoteTime;
  unsigned short int ppqn;
  unsigned short int version;
  unsigned short int numTracks;
  unsigned short int padding;
} MsqHeader;

MsqHeader parse_msq_header(char **file);

typedef struct {
  int magic;
  int version;
  unsigned short int ppqn;
  char quarterNoteTime[3];
  unsigned short int timeSignature;
} CdsHeader;

CdsHeader parse_cds_header(char **file);
