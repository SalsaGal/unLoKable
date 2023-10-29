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