#pragma once

typedef struct musHeader {
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
}

typedef struct msqTable {
		int msqIndex;
		int msqOffset;
}

typedef struct waveEntry {
		char name[20];
		int offset;
		int loopBegin;
		int size;
		int loopEnd;
		int sampleRate;
		int originalPitch; /* to re-define and re-align */
		int loopInfo;
		int sndHandle;
}

typedef struct envelope {
		float delay;
		float attack;
		float hold;
		float decay;
		float sustain;
		float release;
}

typedef struct programZone {
		int pitchFinetuning;
		int reverb;
		float panPosition;
		int keynumHold;
		int keynumDecay;
		struct envelope volumeEnv;
		float volumeEnvAtten;
		float vibDelay;
		float vibFrequency;
		float vibToPitch;
		int rootKey; /* usually padded as 0xFFFFFFFF. Copy the value from the "originalPitch" variable from the "waveEntry" structure */
		char noteLow;
		char noteHigh;
		char velocityLow;
		char velocityHigh;
		int waveIndex;
		float basePriority;
		struct envelope modulEnv;
		float modulEnvToPitch;
}

typedef struct programEntry {
		char name[20];
		int numZones;		
}

typedef struct presetZone {
		int rootKey; /* usually padded as 0xFFFFFFFF. Copy the value from the "rootKey" variable from the "programZone" structure */
		char noteLow;
		char noteHigh;
		char velocityLow;
		char velocityHigh;
		int programIndex;
}

typedef struct presetEntry {
		char name[20];
		int MIDIBankNumber;
		int MIDIPresetNumber;
		int numZones;
}

typedef struct msqHeader {
		int msqID;
		unsigned int quarterNoteTime;
		unsigned short int ppqn;
		unsigned short int version;
		unsigned short int numTracks;
		unsigned short int padding;
}

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