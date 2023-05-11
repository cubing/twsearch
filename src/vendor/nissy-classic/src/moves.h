#ifndef MOVES_H
#define MOVES_H

#include "alg.h"
#include "cube.h"
#include "env.h"

/*
 * Tables are exposed to allow for faster moves in some cases.
 */
extern int              epose_mtable[NMOVES][FACTORIAL12/FACTORIAL8];
extern int              eposs_mtable[NMOVES][FACTORIAL12/FACTORIAL8];
extern int              eposm_mtable[NMOVES][FACTORIAL12/FACTORIAL8];
extern int              eofb_mtable[NMOVES][POW2TO11];
extern int              eorl_mtable[NMOVES][POW2TO11];
extern int              eoud_mtable[NMOVES][POW2TO11];
extern int              cp_mtable[NMOVES][FACTORIAL8];
extern int              coud_mtable[NMOVES][POW3TO7];
extern int              cofb_mtable[NMOVES][POW3TO7];
extern int              corl_mtable[NMOVES][POW3TO7];
extern int              cpos_mtable[NMOVES][FACTORIAL6];

Cube        apply_alg(Alg *alg, Cube cube);
Cube        apply_alg_generic(Alg *alg, Cube c, PieceFilter f, bool a);
Cube        apply_move(Move m, Cube cube);
Alg *       cleanup(Alg *alg);

void        init_moves(void);

#endif
