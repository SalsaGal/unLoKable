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

WaveEntry parse_wave_entry(unsigned char **file) {
  WaveEntry entry;
  for (int i = 0; i < 20; i++) {
    entry.name[i] = parse_byte(file);
  }
	entry.offset = parse_int_be(file);
	entry.loopBegin = parse_int_be(file);
	entry.size = parse_int_be(file);
	entry.loopEnd = parse_int_be(file);
	entry.sampleRate = parse_int_be(file);
	entry.originalPitch = parse_int_be(file); /* to re-define and re-align */
	entry.loopInfo = parse_int_be(file);
	entry.sndHandle = parse_int_be(file);
  return entry;
}
