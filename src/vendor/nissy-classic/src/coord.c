#include "coord.h"

static uint64_t    index_eofb(Cube cube);
static uint64_t    index_eofbepos(Cube cube);
static uint64_t    index_epud(Cube cube);
static uint64_t    index_coud(Cube cube);
static uint64_t    index_corners(Cube cube);
static uint64_t    index_cp(Cube cube);
static uint64_t    index_cphtr(Cube cube);
static uint64_t    index_cornershtr(Cube cube);
static uint64_t    index_cornershtrfin(Cube cube);
static uint64_t    index_drud(Cube cube);
static uint64_t    index_drud_eofb(Cube cube);
static uint64_t    index_htr_drud(Cube cube);
static uint64_t    index_htrfin(Cube cube);
static uint64_t    index_cpud_separate(Cube cube);

static uint64_t    move_eofb(Move m, uint64_t ind);
static uint64_t    move_eofbepos(Move m, uint64_t ind);
static uint64_t    move_epud(Move m, uint64_t ind);
static uint64_t    move_coud(Move m, uint64_t ind);
static uint64_t    move_corners(Move m, uint64_t ind);
static uint64_t    move_cp(Move m, uint64_t ind);
static uint64_t    move_cphtr(Move m, uint64_t ind);
static uint64_t    move_cornershtr(Move m, uint64_t ind);
static uint64_t    move_cornershtrfin(Move m, uint64_t ind);
static uint64_t    move_drud(Move m, uint64_t ind);
static uint64_t    move_drud_eofb(Move m, uint64_t ind);
static uint64_t    move_htr_drud(Move m, uint64_t ind);
static uint64_t    move_htrfin(Move m, uint64_t ind);
static uint64_t    move_cpud_separate(Move m, uint64_t ind);

static void        init_cphtr_cosets(void);
static void        init_cphtr_left_cosets_bfs(int i, int c);
static void        init_cphtr_right_cosets_color(int i, int c);
static void        init_cpud_separate(void);
static void        init_cornershtrfin(void);
static void        init_htr_eposs(void);
static void        init_move_epud(void);
static void        init_move_cphtr(void);


/* All sorts of useful costants and tables  **********************************/

static int              cphtr_left_cosets[FACTORIAL8];
static int              cphtr_right_cosets[FACTORIAL8];
static int              cphtr_right_rep[BINOM8ON4*6];
int                     cpud_separate_ind[FACTORIAL8];
int                     cpud_separate_ant[BINOM8ON4];
static int              cornershtrfin_ind[FACTORIAL8];
int                     cornershtrfin_ant[24*24/6];
static int              htr_eposs_ind[BINOM12ON4];
static int              htr_eposs_ant[BINOM8ON4];
static int              move_cphtr_aux[NMOVES][BINOM8ON4*6];
static int              move_epud_aux[NMOVES][FACTORIAL8];

/* Coordinates and their implementation **************************************/

Coordinate
coord_eofb = {
	.index  = index_eofb,
	.max    = POW2TO11,
	.move   = move_eofb,
};

Coordinate
coord_eofbepos = {
	.index  = index_eofbepos,
	.max    = POW2TO11 * BINOM12ON4,
	.move   = move_eofbepos,
};

Coordinate
coord_coud = {
	.index  = index_coud,
	.max    = POW3TO7,
	.move   = move_coud,
};

Coordinate
coord_corners = {
	.index  = index_corners,
	.max    = POW3TO7 * FACTORIAL8,
	.move   = move_corners,
};

Coordinate
coord_cp = {
	.index  = index_cp,
	.max    = FACTORIAL8,
	.move   = move_cp,
};

Coordinate
coord_cphtr = {
	.index  = index_cphtr,
	.max    = BINOM8ON4 * 6,
	.move   = move_cphtr,
};

Coordinate
coord_cornershtr = {
	.index  = index_cornershtr,
	.max    = POW3TO7 * BINOM8ON4 * 6,
	.move   = move_cornershtr,
};

Coordinate
coord_cornershtrfin = {
	.index  = index_cornershtrfin,
	.max    = 24*24/6,
	.move   = move_cornershtrfin,
};

Coordinate
coord_epud = {
	.index  = index_epud,
	.max    = FACTORIAL8,
	.move   = move_epud,
};

Coordinate
coord_drud = {
	.index  = index_drud,
	.max    = POW2TO11 * POW3TO7 * BINOM12ON4,
	.move   = move_drud,
};

Coordinate
coord_htr_drud = {
	.index  = index_htr_drud,
	.max    = BINOM8ON4 * 6 * BINOM8ON4,
	.move   = move_htr_drud,
};

Coordinate
coord_htrfin = {
	.index  = index_htrfin,
	.max    = 24 * 24 * 24 *24 * 24 / 6, /* should be /12 but it's ok */
	.move   = move_htrfin,
};

Coordinate
coord_drud_eofb = {
	.index  = index_drud_eofb,
	.max    = POW3TO7 * BINOM12ON4,
	.move   = move_drud_eofb,
};

Coordinate
coord_cpud_separate = {
	.index  = index_cpud_separate,
	.max    = BINOM8ON4,
	.move   = move_cpud_separate,
};

/* Indexers ******************************************************************/

static uint64_t
index_eofb(Cube cube)
{
	return cube.eofb;
}

static uint64_t
index_eofbepos(Cube cube)
{
	return (cube.epose / FACTORIAL4) * POW2TO11 + cube.eofb;
}

static uint64_t
index_epud(Cube cube)
{
	uint64_t ret;
	CubeArray *arr = new_cubearray(cube, pf_ep);

	ret = perm_to_index(arr->ep, 8);
	free_cubearray(arr, pf_ep);

	return ret;
}

static uint64_t
index_coud(Cube cube)
{
	return cube.coud;
}

static uint64_t
index_corners(Cube cube)
{
	return cube.coud * FACTORIAL8 + cube.cp;
}

static uint64_t
index_cp(Cube cube)
{
	return cube.cp;
}

static uint64_t
index_cphtr(Cube cube)
{
	return cphtr_right_cosets[cube.cp];
}

static uint64_t
index_cornershtr(Cube cube)
{
	return cube.coud * BINOM8ON4 * 6 + index_cphtr(cube);
}

static uint64_t
index_cornershtrfin(Cube cube)
{
	return cornershtrfin_ind[cube.cp];
}

static uint64_t
index_drud(Cube cube)
{
	uint64_t a, b, c;

	a = cube.eofb;
	b = cube.coud;
	c = cube.epose / FACTORIAL4;

	b *= POW2TO11;
	c *= POW2TO11 * POW3TO7;

	return a + b + c;
}

static uint64_t
index_drud_eofb(Cube cube)
{
	return index_drud(cube) / POW2TO11;
}

static uint64_t
index_htr_drud(Cube cube)
{
	uint64_t a, b;

	a = index_cphtr(cube);
	b = htr_eposs_ind[cube.eposs/24];

	return a * BINOM8ON4 + b;
}

static uint64_t
index_htrfin(Cube cube)
{
	uint64_t epe, eps, epm, cp, ep;

	epe = cube.epose % 24;
	eps = cube.eposs % 24;
	epm = cube.eposm % 24;
	ep = (epe * 24 + eps) *24 + epm;
	cp = index_cornershtrfin(cube);

	return cp * 24 * 24 * 24 + ep;
}

static uint64_t
index_cpud_separate(Cube cube)
{
	return cpud_separate_ind[cube.cp];
}

/* Coordinate movers *********************************************************/

static uint64_t
move_eofb(Move m, uint64_t ind)
{
	return eofb_mtable[m][ind];
}

static uint64_t
move_eofbepos(Move m, uint64_t ind)
{
	uint64_t a, b;

	a = epose_mtable[m][(ind / POW2TO11)*24];
	b = eofb_mtable[m][ind % POW2TO11];

	return (a/24) * POW2TO11 + b;
}

static uint64_t
move_epud(Move m, uint64_t ind)
{
	static int shortlist[NMOVES] = {
		[U] = 0, [U2] = 1, [U3] = 2, [D] = 3, [D2] = 4, [D3] = 5,
		[R2] = 6, [L2] = 7, [F2] = 8, [B2] = 9
	};

	return move_epud_aux[shortlist[m]][ind];
}

static uint64_t
move_coud(Move m, uint64_t ind)
{
	return coud_mtable[m][ind];
}

static uint64_t
move_corners(Move m, uint64_t ind)
{
	uint64_t a, b;

	a = coud_mtable[m][ind / FACTORIAL8];
	b = cp_mtable[m][ind % FACTORIAL8];

	return a * FACTORIAL8 + b;
}

static uint64_t
move_cp(Move m, uint64_t ind)
{
	return cp_mtable[m][ind];
}

static uint64_t
move_cphtr(Move m, uint64_t ind)
{
	return move_cphtr_aux[m][ind];
}

static uint64_t
move_cornershtr(Move m, uint64_t ind)
{
	uint64_t a, b;

	a = coud_mtable[m][ind/(BINOM8ON4 * 6)];
	b = move_cphtr(m, ind % (BINOM8ON4 * 6));

	return a * BINOM8ON4 * 6 + b;
}

static uint64_t
move_cornershtrfin(Move m, uint64_t ind)
{
	int a;

	a = cp_mtable[m][cornershtrfin_ant[ind]];

	return cornershtrfin_ind[a];
}

static uint64_t
move_drud(Move m, uint64_t ind)
{
	uint64_t a, b, c;

	a = eofb_mtable[m][ind % POW2TO11];
	b = coud_mtable[m][(ind / POW2TO11) % POW3TO7];
	c = epose_mtable[m][ind / (POW2TO11 * POW3TO7)];

	return a + (b + c * POW3TO7) * POW2TO11;
}

static uint64_t
move_drud_eofb(Move m, uint64_t ind)
{
	uint64_t a, b;

	a = coud_mtable[m][ind % POW3TO7];
	b = epose_mtable[m][(ind / POW3TO7) * 24] / 24;

	return a + b * POW3TO7;
}

static uint64_t
move_htr_drud(Move m, uint64_t ind)
{
	uint64_t a, b;

	a = move_cphtr(m, ind/BINOM8ON4);
	b = eposs_mtable[m][htr_eposs_ant[ind%BINOM8ON4]];

	return a*BINOM8ON4 + htr_eposs_ind[b/24];
}

static uint64_t
move_htrfin(Move m, uint64_t ind)
{
	uint64_t a, b, bm, bs, be;

	a = move_cornershtrfin(m, ind / (24*24*24));
	bm = eposm_mtable[m][ind%24] % 24;
	bs = eposs_mtable[m][(ind/24)%24] % 24;
	be = epose_mtable[m][(ind/(24*24))%24] % 24;
	b = (be * 24 + bs) * 24 + bm;

	return a * (24*24*24) + b;
}

static uint64_t
move_cpud_separate(Move m, uint64_t ind)
{
	return cpud_separate_ind[cp_mtable[m][cpud_separate_ant[ind]]];
}

/* Init functions implementation *********************************************/

/*
 * There is certainly a better way to do this, but for now I just use
 * a "graph coloring" algorithm to compute the left cosets, and I compose
 * with every possible cp to get the right cosets (it is possible that I am
 * mixing up left and right).
 * 
 * For doing it better "Mathematically", we need 3 things:
 *   - Checking that cp separates the orbits (UFR,UBL,DFL,DBR) and the other
 *     This is easy and it is done in the commented function cphtr_cp().
 *   - Check that there is no ep/cp parity
 *   - Check that we are not in the "3c" case; this is the part I don't
 *     know how to do.
 */
static void
init_cphtr_cosets(void)
{
	unsigned int i; 
	int c = 0, d = 0;

	for (i = 0; i < FACTORIAL8; i++) {
		cphtr_left_cosets[i]  = -1;
		cphtr_right_cosets[i] = -1;
	}

	/* First we compute left cosets with a bfs */
	for (i = 0; i < FACTORIAL8; i++)
		if (cphtr_left_cosets[i] == -1)
			init_cphtr_left_cosets_bfs(i, c++);

	/* Then we compute right cosets using compose() */
	for (i = 0; i < FACTORIAL8; i++)
		if (cphtr_right_cosets[i] == -1)
			init_cphtr_right_cosets_color(i, d++);
}

static void
init_cphtr_left_cosets_bfs(int i, int c)
{
	int j, jj, next[FACTORIAL8], next2[FACTORIAL8], n, n2;

	Move k;

	n = 1;
	next[0] = i;
	cphtr_left_cosets[i] = c;

	while (n != 0) {
		for (j = 0, n2 = 0; j < n; j++) {
			for (k = U2; k < B3; k++) {
				if (!moveset_htr.allowed(k))
					continue;
				jj = apply_move(k, (Cube){ .cp = next[j] }).cp;

				if (cphtr_left_cosets[jj] == -1) {
					cphtr_left_cosets[jj] = c;
					next2[n2++] = jj;
				}
			}
		}

		for (j = 0; j < n2; j++)
			next[j] = next2[j];
		n = n2;
	}
}

static void
init_cphtr_right_cosets_color(int i, int d)
{
	int cp;
	unsigned int j;

	cphtr_right_rep[d] = i;
	for (j = 0; j < FACTORIAL8; j++) {
		if (cphtr_left_cosets[j] == 0) {
			cp = compose_filtered(
			    (Cube){.cp = i}, (Cube){.cp = j}, pf_cp).cp;
			cphtr_right_cosets[cp] = d;
		}
	}
}

static void
init_cpud_separate(void)
{
	unsigned int ui;
	int i, co[8];

	for (ui = 0; ui < FACTORIAL8; ui++) {
		for (i = 0; i < 8; i++)
			co[i] = what_corner_at((Cube){.cp=ui},i)>UBR ?  1 : 0;
		cpud_separate_ind[ui] = subset_to_index(co, 8, 4);
		cpud_separate_ant[cpud_separate_ind[ui]] = ui;
	}
}

static void
init_cornershtrfin(void)
{
	unsigned int i, j;
	int n, c;
	Move m;

	for (i = 0; i < FACTORIAL8; i++)
		cornershtrfin_ind[i] = -1;
	cornershtrfin_ind[0] = 0;

	/* 10-pass, I think 5 is enough, but just in case */
	n = 1;
	for (i = 0; i < 10; i++) {
		for (j = 0; j < FACTORIAL8; j++) {
			if (cornershtrfin_ind[j] == -1)
				continue;
			for (m = U; m < NMOVES; m++) {
				if (moveset_htr.allowed(m)) {
					c = cp_mtable[m][j];
					if (cornershtrfin_ind[c] == -1) {
						cornershtrfin_ind[c] = n;
						cornershtrfin_ant[n] = c;
						n++;
					}
				}
			}
		}
	}
}

void
init_htr_eposs(void)
{
	int ep[12], ep2[12];
	int eps_solved[4] = {UL, UR, DL, DR};
	unsigned int i, j;

	/* NOTE: we loop over all possible positions of S-slice edges,    *
	 * although some of them are not possible (because the E-slice    *
	 * edges are already in the E-slice). Then we discard the invalid *
	 * configurations by checking that the value returned by          *
	 * subset_to_index() is acceptable.                               */
	for (i = 0; i < BINOM12ON4; i++) {
		for (j = 0; j < 12; j++)
			ep[j] = ep2[j] = 0;
		epos_to_partial_ep(i*24, ep, eps_solved);
		for (j = 0; j < 8; j++)
			ep2[j/2 + 4*(j%2)] = ep[j] ? 1 : 0;
		htr_eposs_ind[i] = subset_to_index(ep2, 8, 4);
		if (htr_eposs_ind[i] < (int)BINOM8ON4)
			htr_eposs_ant[htr_eposs_ind[i]] = i*24;
	}
}

static void
init_move_epud(void)
{
	/* TODO: save to file? */
	static int a[12] = { [8] = 8, [9] = 9, [10] = 10, [11] = 11 };
	static int shortlist[NMOVES] = {
		[U] = 0, [U2] = 1, [U3] = 2, [D] = 3, [D2] = 4, [D3] = 5,
		[R2] = 6, [L2] = 7, [F2] = 8, [B2] = 9
	};
	uint64_t ui;
	int j;
	Move mj;
	Cube c;
	CubeArray *arr, *auxarr;

	auxarr = malloc(sizeof(CubeArray));
	auxarr->ep = a;
	for (ui = 0; ui < coord_epud.max; ui++) {
		index_to_perm(ui, 8, a);
		c = arrays_to_cube(auxarr, pf_ep);
		for (j = 0; moveset_drud.sorted_moves[j] != NULLMOVE;
		    j++) {
			mj = moveset_drud.sorted_moves[j];
			arr = new_cubearray(apply_move(mj, c), pf_ep);
			move_epud_aux[shortlist[mj]][ui] =
			    perm_to_index(arr->ep, 8);
			free_cubearray(arr, pf_ep);
		}
	}
	free(auxarr);
}

static void
init_move_cphtr(void)
{
	uint64_t ui;
	Move j;

	for (ui = 0; ui < BINOM8ON4*6; ui++)
		for (j = U; j < NMOVES; j++)
			move_cphtr_aux[j][ui] = cphtr_right_cosets[
			    cp_mtable[j][cphtr_right_rep[ui]]];
}


void
init_coord(void)
{
	static bool initialized = false;
	if (initialized)
		return;
	initialized = true;

	init_trans();

	init_cphtr_cosets();
	init_cornershtrfin();
	init_htr_eposs();
	init_cpud_separate();

	init_move_epud();
	init_move_cphtr();
}

