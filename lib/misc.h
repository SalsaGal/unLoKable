#pragma once

#include "structures.h"

#define BUFFER_SIZE 1024 * 1024

Slice load_buffer(char *path);
char *remove_path(char *path);
char *remove_extension(char *path);
void make_directory(char *path);
void clean_path(char *path);
