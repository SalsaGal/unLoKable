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
