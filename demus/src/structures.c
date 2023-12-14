#include "structures.h"

#include "../../lib/structures.h"

MusHeader parse_mus_header(unsigned char **file) {
  MusHeader to_return;
	to_return.ID = parse_int_le(file);
	to_return.headerSize = parse_int_be(file);
	to_return.versionNumber = parse_int_be(file);
	to_return.reverbVolume = parse_int_be(file);
	to_return.reverbType = parse_int_be(file);
	to_return.reverbMultiply = parse_int_be(file);
	to_return.numSequences = parse_int_be(file);
	to_return.numLabels = parse_int_be(file);
	to_return.offsetToLabelsOffsetsTable = parse_int_be(file);
	to_return.numWaves = parse_int_be(file);
	to_return.numPrograms = parse_int_be(file);
	to_return.numPresets = parse_int_be(file);
  return to_return;
}

MsqTable parse_msq_table(unsigned char **file) {
  MsqTable table;
  table.msqIndex = parse_int_be(file);
  table.msqOffset = parse_int_be(file);
  return table;
}

bool is_valid_char(unsigned char c) {
  switch (c) {
    case 34:
    case 36:
    case 42:
    case 47:
    case 58:
    case 59:
    case 60:
    case 62:
    case 63:
    case 92:
    case 94:
    case 96:
      return false;

    default:
      return (c >= 32) && (c < 127);
  }
}

void parse_name(unsigned char **file, char buffer[20]) {
  bool encountered_garbage = false;
  for (int i = 0; i < 20; i++) {
    unsigned char c = parse_byte(file);
    if (!encountered_garbage && !is_valid_char(c)) {
      encountered_garbage = true;
    }
    buffer[i] = encountered_garbage ? 0 : c;
  }
}

WaveEntry parse_wave_entry(unsigned char **file, bool pc_style) {
  WaveEntry entry;
  parse_name(file, entry.name);
	entry.offset = parse_int_be(file);
	entry.loopBegin = parse_int_be(file);
	entry.size = parse_int_be(file) * (pc_style ? 1 : 2);
	entry.loopEnd = parse_int_be(file);
	entry.sampleRate = parse_int_be(file);
	entry.originalPitch = parse_int_be(file) >> 8;
	entry.loopInfo = parse_int_be(file);
	entry.sndHandle = parse_int_be(file);
  return entry;
}

ProgramEntry parse_program_entry(unsigned char **file) {
  ProgramEntry entry;
  parse_name(file, entry.name);
  entry.numZones = parse_int_be(file);
  return entry;
}

Envelope parse_envelope(unsigned char **file) {
  Envelope envelope;
	envelope.delay = parse_float_be(file);
	envelope.attack = parse_float_be(file);
	envelope.hold = parse_float_be(file);
	envelope.decay = parse_float_be(file);
	envelope.sustain = parse_float_be(file);
	envelope.release = parse_float_be(file);
  return envelope;
}

ProgramZone parse_program_zone(unsigned char **file) {
  ProgramZone zone;
  zone.pitchFinetuning = parse_int_be(file);
  zone.reverb = parse_int_be(file);
  zone.panPosition = parse_float_be(file);
  zone.keynumHold = parse_int_be(file);
  zone.keynumDecay = parse_int_be(file);
  zone.volumeEnv = parse_envelope(file);
  zone.volumeEnvAtten = parse_float_be(file);
  zone.vibDelay = parse_float_be(file);
  zone.vibFrequency = parse_float_be(file);
  zone.vibToPitch = parse_float_be(file);
  zone.rootKey = parse_int_be(file);
  zone.noteLow = parse_byte(file);
  zone.noteHigh = parse_byte(file);
  zone.velocityLow = parse_byte(file);
  zone.velocityHigh = parse_byte(file);
  zone.waveIndex = parse_int_be(file);
  zone.basePriority = parse_float_be(file);
  zone.modulEnv = parse_envelope(file);
  zone.modulEnvToPitch = parse_float_be(file);
  return zone;
}

PresetZone parse_preset_zone(unsigned char **file) {
  PresetZone zone;
  zone.rootKey = parse_int_be(file);
  zone.noteLow = parse_byte(file);
  zone.noteHigh = parse_byte(file);
  zone.velocityLow = parse_byte(file);
  zone.velocityHigh = parse_byte(file);
  zone.programIndex = parse_int_be(file);
  return zone;
}

PresetEntry parse_preset_entry(unsigned char **file) {
  PresetEntry entry;
  parse_name(file, entry.name);
	entry.MIDIBankNumber = parse_int_be(file);
	entry.MIDIPresetNumber = parse_int_be(file);
	entry.numZones = parse_int_be(file);
  return entry;
}
