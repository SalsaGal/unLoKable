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

SndHeader parse_header(char **file);

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
  int msqID;
  unsigned int quarterNoteTime;
  unsigned short int ppqn;
  unsigned short int version;
  unsigned short int numTracks;
  unsigned short int padding;
} MsqHeader;

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

/*
typedef struct {
  char ID[4];
  unsigned int version;
  unsigned int reserved;
  unsigned int dataSize;
  unsigned int sampleRate;
  char padding[12];
  char name[16];
} VagHeader;

typedef struct {
  unsigned char numTones;
  unsigned char volume;
  unsigned char priority;
  unsigned char mode;
  unsigned char panPosition;
  unsigned char reserved;
  unsigned short int attribute;
  unsigned int padding1;
  unsigned int padding2;
} VabProgram;
*/
/*
typedef struct {
  unsigned char priority;
  unsigned char reverbMode;
  unsigned char volume;
  unsigned char panPosition;
  unsigned char rootKey;
    unsigned char pitchFinetuning;
    unsigned char noteLow;
  unsigned char noteHigh;
  unsigned char vibWidth;
  unsigned char vibTime;
  unsigned char porWidth;
  unsigned char porHoldingTime;
  unsigned char minPitchRange;
  unsigned char maxPitchRange;
  unsigned char padding1;
  unsigned char padding2;
  unsigned short int ADSR1;
  unsigned short int ADSR2;
  unsigned short int parentProgram;
  unsigned short int waveIndex;
  unsigned short int padding3;
  unsigned short int padding4;
  unsigned short int padding5;
  unsigned short int padding6;
}
*/
