#ifndef ENV_H
#define ENV_H

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>

extern char *tabledir;

void init_env(void);

#endif
