#ifndef PRUNING_H
#define PRUNING_H

#include "symcoord.h"

extern PruneData        pd_eofb_HTM;
extern PruneData        pd_coud_HTM;
extern PruneData        pd_corners_HTM;
extern PruneData        pd_cornershtr_HTM;
extern PruneData        pd_drud_sym16_HTM;
extern PruneData        pd_drud_eofb;
extern PruneData        pd_drudfin_noE_sym16_drud;
extern PruneData        pd_htr_drud;
extern PruneData        pd_cp_drud;
extern PruneData        pd_htrfin_htr;
extern PruneData        pd_nxopt31_HTM;

extern PruneData *      all_pd[];

void        free_pd(PruneData *pd);
void        genptable(PruneData *pd, int nthreads);
void        print_ptable(PruneData *pd);
uint64_t    ptablesize(PruneData *pd);
int         ptableval(PruneData *pd, Cube cube);

#endif

