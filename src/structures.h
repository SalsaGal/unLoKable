#pragma once

typedef struct sndHeader {
		int ID;
		int headerSize;
		int bankVersion;
		int numPrograms;
		int numZones;
		int numWaves;
		int numSequences;
		int numLabels;
		int reverbMode;
		int reverbDepth;
}

typedef struct sndProgram {
		unsigned short int numZones;
		unsigned short int firstTone;
		unsigned char volume;
		unsigned char panPos;
		unsigned short int padding;

}

typedef struct sndZone {
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