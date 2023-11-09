CC = gcc
CC_FLAGS = -Wall -Wextra -Ilib

LIB_OBJS = $(addprefix out/lib/,structures.o)
SOUND_OBJS = $(addprefix out/sound/,main.o)
SEQUENCE_OBJS = $(addprefix out/sequence/,main.o)

full: out/bin/sequence out/bin/sound

out/bin/sequence: $(LIB_OBJS) $(SEQUENCE_OBJS)
	$(CC) -o $@ $^ $(CC_FLAGS)

out/bin/sound: $(LIB_OBJS) $(SOUND_OBJS)
	$(CC) -o $@ $^ $(CC_FLAGS)

out/sound/%.o: sound/src/%.c
	$(CC) -o $@ $< -c $(CC_FLAGS)

out/sequence/%.o: sequence/src/%.c
	$(CC) -o $@ $< -c $(CC_FLAGS)

out/lib/%.o: lib/%.c
	$(CC) -o $@ $< -c $(CC_FLAGS)
