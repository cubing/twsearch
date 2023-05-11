#ifndef COMMANDS_H
#define COMMANDS_H

#include <time.h>

#include "solve.h"
#include "steps.h"
#include "cubetypes.h"

void                    free_args(CommandArgs *args);
CommandArgs *           new_args(void);

extern Command *        commands[];

extern void twophase_exec_scramble(Alg *scramble);

#endif
