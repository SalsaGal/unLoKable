#pragma once

#include <stdlib.h>

int parse_int(char **file);
unsigned short int parse_word(char **file);
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
  char *start;
  int length;
} Slice;

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
