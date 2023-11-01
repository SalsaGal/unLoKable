#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "structures.h"

#define BUFFER_SIZE 1024 * 1024

int main(int argc, char *argv[]) {
  if (argc != 3) {
    printf("Incorrect argument count\n");
    return 1;
  }

  FILE *snd_file = fopen(argv[1], "rb");
  char *snd_buffer = malloc(BUFFER_SIZE);
  size_t snd_size = fread(snd_buffer, sizeof(char), BUFFER_SIZE, snd_file);

  FILE *smp_file = fopen(argv[2], "rb");
  char *smp_buffer = malloc(BUFFER_SIZE);
  size_t smp_size = fread(smp_buffer, sizeof(char), BUFFER_SIZE, smp_file);

  printf("SND size: 0x%zx\n", snd_size);
  printf("SMP size: 0x%zx\n", smp_size);

  if (snd_size == BUFFER_SIZE || smp_size == BUFFER_SIZE) {
    printf("WARNING: Buffer full, might not have been fully read.\n");
  }

  SndHeader header = parseHeader(&snd_buffer);
  printf("\nHEADER:\n");
  printf("Magic Number: %x\n", header.magicNumber);
  printf("Header Size: %x\n", header.headerSize);
  printf("Bank Version: %x\n", header.bankVersion);
  printf("Num Programs: %x\n", header.numPrograms);
  printf("Num Zones: %x\n", header.numZones);
  printf("Num Waves: %x\n", header.numWaves);
  printf("Num Sequences: %x\n", header.numSequences);
  printf("Num Labels: %x\n", header.numLabels);
  printf("Reverb Mode: %x\n", header.reverbMode);
  printf("Reverb Depth: %x\n", header.reverbDepth);

  for (int i = 0; i < header.numPrograms; i++) {
    SndProgram program = parseProgram(&snd_buffer);
    printf("\nPROGRAM %x:\n", i);
    printf("Num Zones: %x\n", program.numZones);
    printf("First Tone: %x\n", program.firstTone);
    printf("Volume: %x\n", program.volume);
    printf("Pan Pos: %x\n", program.panPos);
  }

  for (int i = 0; i < header.numZones; i++) {
    SndZone zone = parseZone(&snd_buffer);
    printf("\nZONE: %x:\n", i);
    printf("Priority: %x\n", zone.priority);
    printf("Parent Program: %x\n", zone.parentProgram);
    printf("Volume: %x\n", zone.volume);
    printf("Pan Pos: %x\n", zone.panPos);
    printf("Root key: %x\n", zone.rootKey);
    printf("Pitch Finetuning: %x\n", zone.pitchFinetuning);
    printf("Note Low: %x\n", zone.noteLow);
    printf("Note High: %x\n", zone.noteHigh);
    printf("Mode: %x\n", zone.mode);
    printf("Max Pitch Range: %x\n", zone.maxPitchRange);
    printf("ADSR1: %x\n", zone.ADSR1);
    printf("ADSR2: %x\n", zone.ADSR2);
    printf("Wave Index: %x\n", zone.waveIndex);
  }

  printf("\n");
  for (int i = 0; i < header.numWaves; i++) {
    int waveOffset = parseInt(&snd_buffer);
    printf("Wave Offset: %x\n", waveOffset);
  }

  printf("\n");
  for (int i = 0; i < header.numSequences; i++) {
    int sequenceOffset = parseInt(&snd_buffer);
    printf("Sequence Offset: %x\n", sequenceOffset);
  }

  printf("\n");
  for (int i = 0; i < header.numLabels; i++) {
    int labels = parseInt(&snd_buffer);
    printf("Labels: %x\n", labels);
  }

  return 0;
}
