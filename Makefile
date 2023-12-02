ifdef win
CC = x86_64-w64-mingw32-gcc
else
CC = gcc
endif
CC_FLAGS = -Wall -Wextra -Ilib -g -fanalyzer

CDS2SEQ_OBJS = $(addprefix out/cds2seq/,main.o)
LIB_OBJS = $(addprefix out/lib/,structures.o misc.o)
SOUND_OBJS = $(addprefix out/sound/,main.o)
SEQUENCE_OBJS = $(addprefix out/sequence/,main.o)

full: out/bin/cds2seq out/bin/sequence out/bin/sound

out/bin/cds2seq: $(LIB_OBJS) $(CDS2SEQ_OBJS)
	$(CC) -o $@ $^ $(CC_FLAGS)

out/bin/sequence: $(LIB_OBJS) $(SEQUENCE_OBJS)
	$(CC) -o $@ $^ $(CC_FLAGS)

out/bin/sound: $(LIB_OBJS) $(SOUND_OBJS)
	$(CC) -o $@ $^ $(CC_FLAGS)

out/cds2seq/%.o: cds2seq/src/%.c
	$(CC) -o $@ $< -c $(CC_FLAGS)

out/sound/%.o: sound/src/%.c
	$(CC) -o $@ $< -c $(CC_FLAGS)

out/sequence/%.o: sequence/src/%.c
	$(CC) -o $@ $< -c $(CC_FLAGS)

out/lib/%.o: lib/%.c
	$(CC) -o $@ $< -c $(CC_FLAGS)

clean:
	rm out/**/*.o
	rm out/bin/*

setup:
	mkdir out out/bin out/cds2seq out/sound out/sequence out/lib
