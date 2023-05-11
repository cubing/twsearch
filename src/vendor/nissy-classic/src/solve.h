#ifndef SOLVE_H
#define SOLVE_H

#include "moves.h"
#include "steps.h"
#include "trans.h"

AlgList *   solve(Cube cube, Step *step, SolveOptions *opts);
Alg *       solve_2phase(Cube cube, int nthreads);

#endif
