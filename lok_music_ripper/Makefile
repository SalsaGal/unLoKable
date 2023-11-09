CC = gcc
OBJS = $(addprefix out/,main.o)

out/able: $(OBJS)
	$(CC) -o $@ $(OBJS)

out/%.o: src/%.c
	$(CC) -o $@ $< -g -c -Wall -Wextra
