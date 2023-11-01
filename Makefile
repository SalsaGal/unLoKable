CC = gcc
OBJS = $(addprefix out/,main.o structures.o)

out/able: $(OBJS)
	$(CC) -o $@ $(OBJS)

out/%.o: src/%.c
	$(CC) -o $@ $< -g -c -Wall -Wextra

clean:
	rm $(OBJS)
	rm out/able
