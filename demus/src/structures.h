#pragma once

#include <stdbool.h>

typedef struct {
		int ID;
		int headerSize;
		int versionNumber;
		int reverbVolume;
		int reverbType;
		int reverbMultiply;
		int numSequences;
		int numLabels;
		int offsetToLabelsOffsetsTable;
		int numWaves;
		int numPrograms;
		int numPresets;
} MusHeader;

MusHeader parse_mus_header(unsigned char **file);

typedef struct {
		int msqIndex;
		int msqOffset;
} MsqTable;

MsqTable parse_msq_table(unsigned char **file);

typedef struct {
		char name[20];
		int offset;
		int loopBegin;
		int size;
		int loopEnd;
		int sampleRate;
		int originalPitch; /* to re-define and re-align */
		int loopInfo;
		int sndHandle;
} WaveEntry;

WaveEntry parse_wave_entry(unsigned char **file, bool pc_style);

typedef struct {
		float delay;
		float attack;
		float hold;
		float decay;
		float sustain;
		float release;
} Envelope;

Envelope parse_envelope(unsigned char **file);

typedef struct {
		int pitchFinetuning;
		int reverb;
		float panPosition;
		int keynumHold;
		int keynumDecay;
		Envelope volumeEnv;
		float volumeEnvAtten;
		float vibDelay;
		float vibFrequency;
		float vibToPitch;
		int rootKey; /* usually padded as 0xFFFFFFFF. Copy the value from the "originalPitch" variable from the "waveEntry" structure */
		unsigned char noteLow;
		unsigned char noteHigh;
		unsigned char velocityLow;
		unsigned char velocityHigh;
		int waveIndex;
		float basePriority;
		Envelope modulEnv;
		float modulEnvToPitch;
} ProgramZone;

ProgramZone parse_program_zone(unsigned char **file);

typedef struct {
		char name[20];
		int numZones;		
} ProgramEntry;

ProgramEntry parse_program_entry(unsigned char **file);

typedef struct {
		int rootKey; // usually padded as 0xFFFFFFFF. Copy the value from the "rootKey" variable from the "programZone" structure
		unsigned char noteLow;
		unsigned char noteHigh;
		unsigned char velocityLow;
		unsigned char velocityHigh;
		int programIndex;
} PresetZone;

PresetZone parse_preset_zone(unsigned char **file);

typedef struct {
		char name[20];
		int MIDIBankNumber;
		int MIDIPresetNumber;
		int numZones;
} PresetEntry;

PresetEntry parse_preset_entry(unsigned char **file);

typedef struct {
		int msqID;
		unsigned int quarterNoteTime;
		unsigned short int ppqn;
		unsigned short int version;
		unsigned short int numTracks;
		unsigned short int padding;
} MsqHeaderV1_6;

/*
typedef struct vagHeader {
		char ID[4];
		unsigned int version;
		unsigned int reserved;
		unsigned int dataSize;
		unsigned int sampleRate;
		char padding[12];
		char name[16];
}

typedef struct vabProgramBody {
		unsigned char numTones;
		unsigned char volume;
		unsigned char priority;
		unsigned char mode;
		unsigned char panPosition;
		unsigned char reserved;
		unsigned short int attribute;
		unsigned int padding1;
		unsigned int padding2;
}
*/
/*
typedef struct vabZone {
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
