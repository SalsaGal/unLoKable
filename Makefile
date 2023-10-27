OBJS = $(addprefix out/,main.o)

out/able: $(OBJS)
	clang -o $@ $(OBJS)

out/%.o: src/%.c
	clang -o $@ $< -g -c
