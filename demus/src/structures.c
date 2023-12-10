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

WaveEntry parse_wave_entry(unsigned char **file, bool pc_style) {
  WaveEntry entry;
  bool encountered_garbage = false;
  for (int i = 0; i < 20; i++) {
    unsigned char c = parse_byte(file);
    if (!encountered_garbage && !is_valid_char(c)) {
      encountered_garbage = true;
    }
    entry.name[i] = encountered_garbage ? 0 : c;
  }
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
