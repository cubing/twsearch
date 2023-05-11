#ifndef TRANS_H
#define TRANS_H

#include "moves.h"

/*
 * Tables are exposed to allow faster partial transformations in some
 * specific cases (in symcoord)
 */
extern int               epose_ttable[NTRANS][FACTORIAL12/FACTORIAL8];
extern int               eposs_ttable[NTRANS][FACTORIAL12/FACTORIAL8];
extern int               eposm_ttable[NTRANS][FACTORIAL12/FACTORIAL8];
extern int               eo_ttable[NTRANS][POW2TO11];
extern int               cp_ttable[NTRANS][FACTORIAL8];
extern int               co_ttable[NTRANS][POW3TO7];
extern int               cpos_ttable[NTRANS][FACTORIAL6];
extern Move              moves_ttable[NTRANS][NMOVES];

Cube        apply_trans(Trans t, Cube cube);
Trans       inverse_trans(Trans t);
Alg *       rotation_alg(Trans i);
void        transform_alg(Trans i, Alg *alg);

void        init_trans(void);

#endif
