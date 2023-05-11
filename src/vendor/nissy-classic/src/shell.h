#ifndef SHELL_H
#define SHELL_H

#include "commands.h"

#define MAXLINELEN          10000
#define MAXTOKENLEN         255
#define MAXNTOKENS          255

bool        checkfiles(void);
void        exec_args(int c, char **v);
void        launch(bool batchmode);

#endif
