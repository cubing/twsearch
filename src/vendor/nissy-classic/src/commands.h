#ifndef COMMANDS_H
#define COMMANDS_H

#include <time.h>

#include "solve.h"
#include "steps.h"

void                    free_args(CommandArgs *args);
CommandArgs *           new_args(void);

extern Command *        commands[];

#endif
