#ifndef ALG_H
#define ALG_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "cubetypes.h"
#include "utils.h"

extern Moveset          moveset_HTM;
extern Moveset          moveset_URF;
extern Moveset          moveset_eofb;
extern Moveset          moveset_drud;
extern Moveset          moveset_htr;

void        append_alg(AlgList *l, Alg *alg);
void        append_move(Alg *alg, Move m, bool inverse);
Move        base_move(Move m);
void        compose_alg(Alg *alg1, Alg *alg2);
bool        commute(Move m1, Move m2);
void        copy_alg(Alg *src, Alg *dst);
void        free_alg(Alg *alg);
void        free_alglist(AlgList *l);
void        inplace(Alg * (*f)(Alg *), Alg *alg);
Alg *       inverse_alg(Alg *alg);
Move        inverse_move(Move m);
char *      move_string(Move m);
void        movelist_to_position(Move *ml, int *pos);
void        moveset_to_list(Moveset ms, Move *lst);
Alg *       new_alg(char *str);
AlgList *   new_alglist(void);
Alg *       on_inverse(Alg *alg);
void        print_alg(Alg *alg, bool l);
void        print_alglist(AlgList *al, bool l);
void        swapmove(Move *m1, Move *m2);
Alg *       unniss(Alg *alg);

void        init_moveset(Moveset *ms);
void        init_all_movesets(void);

#endif

