#ifndef SYMCOORD_H
#define SYMCOORD_H

#include "coord.h"

extern Coordinate       coord_cp_sym16;
extern Coordinate       coord_eofbepos_sym16;
extern Coordinate       coord_drud_sym16;
extern Coordinate       coord_drudfin_noE_sym16;
extern Coordinate       coord_nxopt31;

extern SymData          *all_sd[];

void                    free_sd(SymData *sd);
void                    init_symcoord(void);

#endif
