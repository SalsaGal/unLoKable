#include "../../lib/misc.h"
#include "../../lib/structures.h"
#include "../../lib/strings.h"
#include "structures.h"

#include <getopt.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void write_byte(FILE *file, unsigned char byte) {
  fprintf(file, "%c", byte);
}

void write_byte_count(FILE *file, unsigned char byte, int count) {
  for (int i = 0; i < count; i++) {
    write_byte(file, byte);
  }
}

int secs_to_timecent(float seconds) {
  return (int) 1200.0 * log2f(seconds < 0.001 ? 0.001 : seconds);
}

int semitone_tuning(int note) {
  return note / 256;
}

int cents_tuning(int note) {
  return (note % 256) * 100 / 256;
}

int pan_convert(float pan) {
  return (int) (pan * 1000 - 500);
}

int percentage_to_decibel(float sustain) {
  return (int) (sustain * 10);
}

int main(int argc, char *argv[]) {
  char *output_dir = NULL;
  bool pc_style = true;
  int opt;
  while ((opt = getopt(argc, argv, "hcpo:")) != -1) {
    switch (opt) {
    case 'h':
      printf("TODO Help");
      return 0;

    case 'c':
      pc_style = false;
      break;

    case 'p':
      pc_style = true;
      break;

    case 'o':
      output_dir = optarg;
      break;
    }
  }

  if (optind >= argc - 1) {
    printf(ERROR_MISSING_ARGS);
    return 1;
  }

  char *mus_path = argv[optind++];
  char *sam_path = argv[optind++];

  Slice mus_buffer = load_buffer(mus_path);
  Slice sam_buffer = load_buffer(sam_path);

  unsigned char *mus_buffer_cursor = mus_buffer.start;

  MusHeader header = parse_mus_header(&mus_buffer_cursor);
  if (header.ID != 0x4D757321) {
    printf("Incorrect magic number!\n");
    return EXIT_FAILURE;
  }
	printf("ID: %d\n", header.ID);
	printf("headerSize: %d\n", header.headerSize);
	printf("versionNumber: %d\n", header.versionNumber);
	printf("reverbVolume: %d\n", header.reverbVolume);
	printf("reverbType: %d\n", header.reverbType);
	printf("reverbMultiply: %d\n", header.reverbMultiply);
	printf("numSequences: %d\n", header.numSequences);
	printf("numLabels: %d\n", header.numLabels);
	printf("offsetToLabelsOffsetsTable: %d\n", header.offsetToLabelsOffsetsTable);
	printf("numWaves: %d\n", header.numWaves);
	printf("numPrograms: %d\n", header.numPrograms);
	printf("numPresets: %d\n", header.numPresets);
  MsqTable *msq_tables = calloc(header.numSequences, sizeof(MsqTable));
  for (int i = 0; i < header.numSequences; i++) {
    msq_tables[i] = parse_msq_table(&mus_buffer_cursor);
    printf("MSQ table #%d: %x, %x\n", i, msq_tables[i].msqIndex, msq_tables[i].msqOffset);
  }
  int *layers = calloc(header.numPresets + header.numPrograms, sizeof(int));
  for (int i = 0; i < header.numPresets + header.numPrograms; i++) {
    layers[i] = parse_int_be(&mus_buffer_cursor);
    printf("Layer #%d: %x\n", i, layers[i]);
  }
  WaveEntry *wave_entries = calloc(header.numWaves, sizeof(WaveEntry));
  for (int i = 0; i < header.numWaves; i++) {
    wave_entries[i] = parse_wave_entry(&mus_buffer_cursor);
    printf("Wave entry #%d \"%.20s\":\n", i, wave_entries[i].name);
		printf("offset: %x\n", wave_entries[i].offset);
		printf("loopBegin: %x\n", wave_entries[i].loopBegin);
		printf("size: %x\n", wave_entries[i].size);
		printf("loopEnd: %x\n", wave_entries[i].loopEnd);
		printf("sampleRate: %x\n", wave_entries[i].sampleRate);
		printf("originalPitch: %x\n", wave_entries[i].originalPitch);
		printf("loopInfo: %x\n", wave_entries[i].loopInfo);
		printf("sndHandle: %x\n", wave_entries[i].sndHandle);
  }
  ProgramEntry *program_entries = calloc(header.numPrograms, sizeof(ProgramEntry));
  ProgramZone **program_zones = calloc(header.numPrograms, sizeof(ProgramZone *));
  for (int i = 0; i < header.numPrograms; i++) {
    // Load program entry
    program_entries[i] = parse_program_entry(&mus_buffer_cursor);
    printf("Program entry \"%s\", %d\n", program_entries[i].name, program_entries[i].numZones);

    // Load program zone
    program_zones[i] = calloc(program_entries[i].numZones, sizeof(ProgramZone));
    for (int j = 0; j < program_entries[i].numZones; j++) {
      program_zones[i][j] = parse_program_zone(&mus_buffer_cursor);
      printf("Program zone set #%d, zone #%d\n", i, j);
  		printf("\tpitchFinetuning: %d\n", program_zones[i][j].pitchFinetuning);
  		printf("\treverb: %d\n", program_zones[i][j].reverb);
  		printf("\tpanPosition: %f\n", program_zones[i][j].panPosition);
  		printf("\tkeynumHold: %d\n", program_zones[i][j].keynumHold);
  		printf("\tkeynumDecay: %d\n", program_zones[i][j].keynumDecay);
      printf("\tmodulEnvToPitch: %f\n", program_zones[i][j].modulEnvToPitch);
  		printf("\tenvelope.delay: %f\n", program_zones[i][j].volumeEnv.delay);
  		printf("\tenvelope.attack: %f\n", program_zones[i][j].volumeEnv.attack);
    }
  }

  PresetEntry *preset_entries = calloc(header.numPresets, sizeof(PresetEntry));
  PresetZone **preset_zones = calloc(header.numPresets, sizeof(PresetZone *));
  for (int i = 0; i < header.numPresets; i++) {
    // Load preset entry
    preset_entries[i] = parse_preset_entry(&mus_buffer_cursor);
    printf("Preset zone \"%s\", %d\n", preset_entries[i].name, preset_entries[i].numZones);
    printf("\tZone count: %d\n", preset_entries[i].numZones);

    // Load preset zone
    preset_zones[i] = calloc(preset_entries[i].numZones, sizeof(PresetZone));
    for (int j = 0; j < preset_entries[i].numZones; j++) {
      printf("Preset zone set #%d, zone #%d\n", i, j);
      preset_zones[i][j] = parse_preset_zone(&mus_buffer_cursor);
  		printf("\tnoteLow: %x\n", preset_zones[i][j].noteLow);
  		printf("\tnoteHigh: %x\n", preset_zones[i][j].noteHigh);
  		printf("\tvelocityLow: %x\n", preset_zones[i][j].velocityLow);
  		printf("\tvelocityHigh: %x\n", preset_zones[i][j].velocityHigh);
  		printf("\tprogramIndex: %x\n", preset_zones[i][j].programIndex);
    }
  }

  Slice *sequences = calloc(header.numSequences, sizeof(Slice));
  for (int i = 0; i < header.numSequences; i++) {
    sequences[i].start = mus_buffer.start + msq_tables[i].msqOffset;
    if (i > 0) {
      sequences[i - 1].length = sequences[i].start - sequences[i - 1].start;
    }
  }
  sequences[header.numSequences - 1].length = (mus_buffer.start + header.offsetToLabelsOffsetsTable) - sequences[header.numSequences - 1].start;

  for (int i = 0; i < header.numSequences; i++) {
    printf("Sequence #%i: 0x%x + 0x%x\n", i, sequences[i].start, sequences[i].length);
  }

  make_directory(remove_extension(mus_path));
  char *sequences_path = calloc(strlen(remove_extension(mus_path)) + 10, sizeof(char));
  sprintf(sequences_path, "%s/sequences", remove_extension(mus_path));
  clean_path(sequences_path);
  make_directory(sequences_path);

  for (int i = 0; i < header.numSequences; i++) {
    // overland_0003.msq
    char *msq_path = calloc(strlen(remove_extension(mus_path)) + strlen(remove_extension(remove_path(mus_path))) + 20, sizeof(char));
    sprintf(msq_path, "%s/sequences/%s_%04d.msq", remove_extension(mus_path), remove_extension(remove_path(mus_path)), i);

    FILE *msq_out = fopen(msq_path, "wb");
    for (unsigned char *c = sequences[i].start; c < sequences[i].start + sequences[i].length; c++) {
      write_byte(msq_out, *c);
    }
    fclose(msq_out);
  }

  Slice *waves = calloc(header.numWaves, sizeof(Slice));
  for (int i = 0; i < header.numWaves; i++) {
    waves[i].start = sam_buffer.start + wave_entries[i].offset;
    waves[i].length = wave_entries[i].size;
    printf("Wave #%d: 0x%x + 0x%x\n", i, waves[i].start, waves[i].length);

    if (!pc_style) {
      unsigned char *to_check = waves[i].start + waves[i].length - 16;
      // TODO Fix this
      bool to_change = true;
      if (to_check[0] != 0x07 && to_check[1] != 00) {
        to_change = false;
      }
      for (int i = 2; i < 16; i++) {
        if (to_check[i] != 0x77) {
          to_change = false;
          break;
        }
      }

      if (to_change) {
        to_check[1] = 0x07;
      }
    }
  }

  char *samples_path = calloc(strlen(remove_extension(mus_path)) + 8, sizeof(char));
  sprintf(samples_path, "%s/samples", remove_extension(mus_path));
  clean_path(samples_path);
  make_directory(samples_path);

  for (int i = 0; i < header.numWaves; i++) {
    char *msq_path = calloc(strlen(remove_extension(mus_path)) + strlen(remove_extension(remove_path(mus_path))) + 16 + strlen(wave_entries[i].name), sizeof(char));
    sprintf(msq_path, "%s/samples/%.20s.ads", remove_extension(mus_path), wave_entries[i].name);

    FILE *ads_out = fopen(msq_path, "wb");
    write_byte(ads_out, 0x53);
    write_byte(ads_out, 0x53);
    write_byte(ads_out, 0x68);
    write_byte(ads_out, 0x64);

    write_byte(ads_out, 0x18);
    write_byte_count(ads_out, 0, 3);

    if (pc_style) {
      write_byte(ads_out, 0x01);
    } else {
      write_byte(ads_out, 0x10);
    }
    write_byte_count(ads_out, 0, 3);

    write_byte(ads_out, (wave_entries[i].sampleRate & 0xff));
    write_byte(ads_out, (wave_entries[i].sampleRate & 0xff00) >> 8);
    write_byte(ads_out, (wave_entries[i].sampleRate & 0xff0000) >> 16);
    write_byte(ads_out, (wave_entries[i].sampleRate & 0xff000000) >> 24);

    write_byte(ads_out, 1);
    write_byte_count(ads_out, 0, 3);

    write_byte_count(ads_out, 0, 4);

    write_byte_count(ads_out, 0xff, 8);

    write_byte(ads_out, 0x53);
    write_byte(ads_out, 0x53);
    write_byte(ads_out, 0x62);
    write_byte(ads_out, 0x64);

    write_byte(ads_out, (wave_entries[i].size & 0xff));
    write_byte(ads_out, (wave_entries[i].size & 0xff00) >> 8);
    write_byte(ads_out, (wave_entries[i].size & 0xff0000) >> 16);
    write_byte(ads_out, (wave_entries[i].size & 0xff000000) >> 24);
    
    for (unsigned char *c = waves[i].start; c < waves[i].start + waves[i].length; c++) {
      write_byte(ads_out, *c);
    }
    fclose(ads_out);
  }

  char *smp_loop_info_path = calloc(strlen(mus_path) * 8, sizeof(char));
  sprintf(smp_loop_info_path, "%s/%s_smploopinfo.txt", remove_extension(mus_path), remove_extension(remove_path(mus_path)));
  FILE *smp_loop_info = fopen(smp_loop_info_path, "w");
  for (int i = 0; i < header.numWaves; i++) {
    fprintf(smp_loop_info, "%d %d %.20s.wav\n", wave_entries[i].loopBegin, wave_entries[i].loopEnd, wave_entries[i].name);
  }
  fclose(smp_loop_info);

  char *info_path = calloc(strlen(mus_path) * 8, sizeof(char));
  sprintf(info_path, "%s/%s.txt", remove_extension(mus_path), remove_extension(remove_path(mus_path)));
  FILE *info_file = fopen(info_path, "w");

  fprintf(info_file, "[Samples]\r\n");
  for (int i = 0; i < header.numWaves; i++) {
    fprintf(info_file, "\r\n    SampleName=%.20s\r\n", wave_entries[i].name);
    fprintf(info_file, "        SampleRate=%d\r\n", wave_entries[i].sampleRate);
    fprintf(info_file, "        Key=%d\r\n", wave_entries[i].originalPitch);
    fprintf(info_file, "        FineTune=0\r\n");
    fprintf(info_file, "        Type=1\r\n");
  }

  fprintf(info_file, "\r\n\r\n[Instruments]\r\n");
  for (int i = 0; i < header.numPrograms; i++) {
    fprintf(info_file, "\r\n    InstrumentName=%.20s\r\n", program_entries[i].name);

    for (int j = 0; j < program_entries[i].numZones; j++) {
      fprintf(info_file, "\r\n        Sample=%.20s\r\n", wave_entries[program_zones[i][j].waveIndex].name);
      fprintf(info_file, "            Z_coarseTune=%d\r\n", semitone_tuning(program_zones[i][j].pitchFinetuning));
      fprintf(info_file, "            Z_fineTune=%d\r\n", cents_tuning(program_zones[i][j].pitchFinetuning));
      fprintf(info_file, "            Z_reverbEffectsSend=%d\r\n", program_zones[i][j].reverb * 10);
      fprintf(info_file, "            Z_pan=%d\r\n", pan_convert(program_zones[i][j].panPosition));
      fprintf(info_file, "            Z_keynumToVolEnvHold=%d\r\n", program_zones[i][j].keynumHold);
      fprintf(info_file, "            Z_keynumToVolEnvDecay=%d\r\n", program_zones[i][j].keynumDecay);
      fprintf(info_file, "            Z_attackVolEnv=%d\r\n", secs_to_timecent(program_zones[i][j].volumeEnv.attack));
      fprintf(info_file, "            Z_decayVolEnv=%d\r\n", secs_to_timecent(program_zones[i][j].volumeEnv.decay));
      fprintf(info_file, "            Z_sustainVolEnv=%d\r\n", percentage_to_decibel(program_zones[i][j].volumeEnv.sustain));
      fprintf(info_file, "            Z_releaseVolEnv=%d\r\n", secs_to_timecent(program_zones[i][j].volumeEnv.release));
      fprintf(info_file, "            Z_initialAttenuation=%d\r\n", (int) (program_zones[i][j].volumeEnvAtten * 10.0));
      fprintf(info_file, "            Z_delayVibLFO=%d\r\n", secs_to_timecent(program_zones[i][j].vibDelay));
      fprintf(info_file, "            Z_freqVibLFO=%d\r\n", secs_to_timecent(program_zones[i][j].vibFrequency / 8.176));
      fprintf(info_file, "            Z_vibLfoToPitch=%d\r\n", (int) program_zones[i][j].vibToPitch);
      fprintf(info_file, "            Z_LowKey=%d\r\n", program_zones[i][j].noteLow);
      fprintf(info_file, "            Z_HighKey=%d\r\n", program_zones[i][j].noteHigh);
      fprintf(info_file, "            Z_LowVelocity=%d\r\n", program_zones[i][j].velocityLow);
      fprintf(info_file, "            Z_HighVelocity=%d\r\n", program_zones[i][j].velocityHigh);
      fprintf(info_file, "            Z_attackModEnv=%d\r\n", secs_to_timecent(program_zones[i][j].modulEnv.attack));
      fprintf(info_file, "            Z_decayModEnv=%d\r\n", secs_to_timecent(program_zones[i][j].modulEnv.decay));
      fprintf(info_file, "            Z_sustainModEnv=%d\r\n", secs_to_timecent(program_zones[i][j].modulEnv.sustain));
      fprintf(info_file, "            Z_releaseModEnv=%d\r\n", secs_to_timecent(program_zones[i][j].modulEnv.release));
      fprintf(info_file, "            Z_modEnvToPitch=%d\r\n", (int) program_zones[i][j].modulEnvToPitch);
    }

    fprintf(info_file, "\r\n        GlobalZone\r\n\r\n");
  }

  fprintf(info_file, "\r\n[Presets]");
  for (int i = 0; i < header.numPresets; i++) {
    fprintf(info_file, "\r\n\r\n    PresetName=%.20s\r\n", preset_entries[i].name);
    fprintf(info_file, "        Bank=%d\r\n", preset_entries[i].MIDIBankNumber);
    fprintf(info_file, "        Program=%d\r\n\r\n", preset_entries[i].MIDIPresetNumber);

    for (int j = 0; j < preset_entries[i].numZones; j++) {
      fprintf(info_file, "        Instrument=%.20s\r\n", program_entries[preset_zones[i][j].programIndex].name);
      fprintf(info_file, "            L_LowKey=%d\r\n", preset_zones[i][j].noteLow);
      fprintf(info_file, "            L_HighKey=%d\r\n", preset_zones[i][j].noteHigh);
      fprintf(info_file, "            L_LowVelocity=%d\r\n", preset_zones[i][j].velocityLow);
      fprintf(info_file, "            L_HighVelocity=%d\r\n", preset_zones[i][j].velocityHigh);
      fprintf(info_file, "\r\n        GlobalLayer\r\n");
    }
  }

  fprintf(info_file, "\r\n\r\n[Info]\r\n");
  fprintf(info_file, "Version=2.1\r\n");
  fprintf(info_file, "Engine=EMU8000\r\n");
  fprintf(info_file, "Name=%s\r\n", remove_extension(remove_path(mus_path)));
  fprintf(info_file, "Editor=Demus\r\n");

  fclose(info_file);

  return EXIT_SUCCESS;
}
