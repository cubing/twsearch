#include "cube.h"

/* Local functions ***********************************************************/

static void             init_inverse(void);
static bool             read_invtables_file(void);
static bool             write_invtables_file(void);

/* Tables ********************************************************************/

static uint16_t         eo_invtable_e[POW2TO11][BINOM12ON4*FACTORIAL4];
static uint16_t         eo_invtable_s[POW2TO11][BINOM12ON4*FACTORIAL4];
static uint16_t         eo_invtable_m[POW2TO11][BINOM12ON4*FACTORIAL4];
static uint16_t         co_invtable[POW3TO7][FACTORIAL8];
static uint16_t         cp_invtable[FACTORIAL8];
static uint16_t         cpos_invtable[FACTORIAL6];

/* Functions implementation **************************************************/

int
array_ep_to_epos(int *ep, int *ss)
{
	int epos[12] = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
	int eps[4];
	int i, j, is;

	for (i = 0, is = 0; i < 12; i++) {
		for (j = 0; j < 4; j++) {
			if (ep[i] == ss[j]) {
				eps[is++] = j;
				epos[i] = 1;
			}
		}
	}

	for (i = 0; i < 4; i++)
		swap(&epos[ss[i]], &epos[i+8]);

	return 24 * subset_to_index(epos, 12, 4) + perm_to_index(eps, 4);
}

Cube
arrays_to_cube(CubeArray *arr, PieceFilter f)
{
	Cube ret = {0};

	static int epe_solved[4] = {FR, FL, BL, BR};
	static int eps_solved[4] = {UL, UR, DL, DR};
	static int epm_solved[4] = {UF, UB, DF, DB};

	if (f.epose)
		ret.epose = array_ep_to_epos(arr->ep, epe_solved);
	if (f.eposs)
		ret.eposs = array_ep_to_epos(arr->ep, eps_solved);
	if (f.eposm)
		ret.eposm = array_ep_to_epos(arr->ep, epm_solved);
	if (f.eofb)
		ret.eofb = digit_array_to_int(arr->eofb, 11, 2);
	if (f.eorl)
		ret.eorl = digit_array_to_int(arr->eorl, 11, 2);
	if (f.eoud)
		ret.eoud = digit_array_to_int(arr->eoud, 11, 2);
	if (f.cp)
		ret.cp = perm_to_index(arr->cp, 8);
	if (f.coud)
		ret.coud = digit_array_to_int(arr->coud, 7, 3);
	if (f.corl)
		ret.corl = digit_array_to_int(arr->corl, 7, 3);
	if (f.cofb)
		ret.cofb = digit_array_to_int(arr->cofb, 7, 3);
	if (f.cpos)
		ret.cpos = perm_to_index(arr->cpos, 6);

	return ret;
}

Cube
compose_filtered(Cube c2, Cube c1, PieceFilter f)
{
	CubeArray *arr = new_cubearray(c2, f);
	Cube ret;

	ret = move_via_arrays(arr, c1, f);
	free_cubearray(arr, f);

	return ret;
}

void
cube_to_arrays(Cube cube, CubeArray *arr, PieceFilter f)
{
	int i;

	static int epe_solved[4] = {FR, FL, BL, BR};
	static int eps_solved[4] = {UL, UR, DL, DR};
	static int epm_solved[4] = {UF, UB, DF, DB};

	if (f.epose || f.eposs || f.eposm)
		for (i = 0; i < 12; i++)
			arr->ep[i] = -1;

	if (f.epose) 
		epos_to_partial_ep(cube.epose, arr->ep, epe_solved); 
	if (f.eposs) 
		epos_to_partial_ep(cube.eposs, arr->ep, eps_solved); 
	if (f.eposm)
		epos_to_partial_ep(cube.eposm, arr->ep, epm_solved); 
	if (f.eofb)
		int_to_sum_zero_array(cube.eofb, 2, 12, arr->eofb);
	if (f.eorl)
		int_to_sum_zero_array(cube.eorl, 2, 12, arr->eorl);
	if (f.eoud)
		int_to_sum_zero_array(cube.eoud, 2, 12, arr->eoud);
	if (f.cp)
		index_to_perm(cube.cp, 8, arr->cp);
	if (f.coud)
		int_to_sum_zero_array(cube.coud, 3, 8, arr->coud);
	if (f.corl)
		int_to_sum_zero_array(cube.corl, 3, 8, arr->corl);
	if (f.cofb)
		int_to_sum_zero_array(cube.cofb, 3, 8, arr->cofb);
	if (f.cpos)
		index_to_perm(cube.cpos, 6, arr->cpos);
}

void
epos_to_compatible_ep(int epos, int *ep, int *ss)
{
	int i, j, k, other[8];
	bool flag;

	for (i = 0; i < 12; i++)
		ep[i] = -1;

	epos_to_partial_ep(epos, ep, ss);

	for (i = 0, j = 0; i < 12; i++) {
		flag = false;
		for (k = 0; k < 4; k++)
			flag = flag || (i == ss[k]);
		if (!flag)
			other[j++] = i;
	}
		
	for (i = 0, j = 0; i < 12; i++)
		if (ep[i] == -1)
			ep[i] = other[j++];
}

void
epos_to_partial_ep(int epos, int *ep, int *ss)
{
	int i, is, eposs[12], eps[4];

	index_to_perm(epos % FACTORIAL4, 4, eps);
	index_to_subset(epos / FACTORIAL4, 12, 4, eposs);

	for (i = 0; i < 4; i++)
		swap(&eposs[ss[i]], &eposs[i+8]);

	for (i = 0, is = 0; i < 12; i++)
		if (eposs[i])
			ep[i] = ss[eps[is++]];
}

void
fix_eorleoud(CubeArray *arr)
{
	int i;

	for (i = 0; i < 12; i++) {
		if ((edge_slice(i) == 0 && edge_slice(arr->ep[i]) != 0) ||
		    (edge_slice(i) != 0 && edge_slice(arr->ep[i]) == 0)) {
			arr->eorl[i] = 1 - arr->eofb[i];
		} else {
			arr->eorl[i] = arr->eofb[i];
		}

		if ((edge_slice(i) == 2 && edge_slice(arr->ep[i]) != 2) ||
		    (edge_slice(i) != 2 && edge_slice(arr->ep[i]) == 2)) {
			arr->eoud[i] = 1 - arr->eofb[i];
		} else {
			arr->eoud[i] = arr->eofb[i];
		}
	}
}

void
fix_cofbcorl(CubeArray *arr)
{
	int i;

	for (i = 0; i < 8; i++) {
		if (i % 2 == arr->cp[i] % 2) {
			arr->cofb[i] = arr->coud[i];
			arr->corl[i] = arr->coud[i];
		} else {
			if (arr->cp[i] % 2 == 0) {
				arr->cofb[i] = (arr->coud[i]+1)%3;
				arr->corl[i] = (arr->coud[i]+2)%3;
			} else {
				arr->cofb[i] = (arr->coud[i]+2)%3;
				arr->corl[i] = (arr->coud[i]+1)%3;
			}
		}
	}
}

Cube
fourval_to_cube(int eofb, int ep, int coud, int cp)
{
	CubeArray *arr;

	arr = new_cubearray((Cube){0}, pf_all);

	index_to_perm(ep, 12, arr->ep);
	index_to_perm(cp,  8, arr->cp);
	int_to_sum_zero_array(eofb, 2, 12, arr->eofb);
	int_to_sum_zero_array(coud, 3,  8, arr->coud);

	/* fix parity */
	if (perm_sign(arr->ep, 12) != perm_sign(arr->cp, 8))
		swap(&(arr->ep[0]), &(arr->ep[1]));

	fix_eorleoud(arr);
	fix_cofbcorl(arr);

	return arrays_to_cube(arr, pf_all);
}

void
free_cubearray(CubeArray *arr, PieceFilter f)
{
	if (f.epose || f.eposs || f.eposm)
		free(arr->ep);
	if (f.eofb)
		free(arr->eofb);
	if (f.eorl)
		free(arr->eorl);
	if (f.eoud)
		free(arr->eoud);
	if (f.cp)
		free(arr->cp);
	if (f.coud)
		free(arr->coud);
	if (f.corl)
		free(arr->corl);
	if (f.cofb)
		free(arr->cofb);
	if (f.cpos)
		free(arr->cpos);

	free(arr);
}

Cube
move_via_arrays(CubeArray *arr, Cube c, PieceFilter f)
{
	CubeArray *arrc = new_cubearray(c, f);
	Cube ret;

	if (f.epose || f.eposs || f.eposm)
		apply_permutation(arr->ep, arrc->ep, 12);

	if (f.eofb) {
		apply_permutation(arr->ep, arrc->eofb, 12);
		sum_arrays_mod(arr->eofb, arrc->eofb, 12, 2);
	}

	if (f.eorl) {
		apply_permutation(arr->ep, arrc->eorl, 12);
		sum_arrays_mod(arr->eorl, arrc->eorl, 12, 2);
	}

	if (f.eoud) {
		apply_permutation(arr->ep, arrc->eoud, 12);
		sum_arrays_mod(arr->eoud, arrc->eoud, 12, 2);
	}

	if (f.cp)
		apply_permutation(arr->cp, arrc->cp, 8);

	if (f.coud) {
		apply_permutation(arr->cp, arrc->coud, 8);
		sum_arrays_mod(arr->coud, arrc->coud, 8, 3);
	}

	if (f.corl) {
		apply_permutation(arr->cp, arrc->corl, 8);
		sum_arrays_mod(arr->corl, arrc->corl, 8, 3);
	}

	if (f.cofb) {
		apply_permutation(arr->cp, arrc->cofb, 8);
		sum_arrays_mod(arr->cofb, arrc->cofb, 8, 3);
	}

	if (f.cpos)
		apply_permutation(arr->cpos, arrc->cpos, 6);

	ret = arrays_to_cube(arrc, f);
	free_cubearray(arrc, f);

	return ret;
}

CubeArray *
new_cubearray(Cube cube, PieceFilter f)
{
	CubeArray *arr = malloc(sizeof(CubeArray));

	if (f.epose || f.eposs || f.eposm)
		arr->ep   = malloc(12 * sizeof(int));
	if (f.eofb)
		arr->eofb = malloc(12 * sizeof(int));
	if (f.eorl)
		arr->eorl = malloc(12 * sizeof(int));
	if (f.eoud)
		arr->eoud = malloc(12 * sizeof(int));
	if (f.cp)
		arr->cp   = malloc(8  * sizeof(int));
	if (f.coud)
		arr->coud = malloc(8  * sizeof(int));
	if (f.corl)
		arr->corl = malloc(8  * sizeof(int));
	if (f.cofb)
		arr->cofb = malloc(8  * sizeof(int));
	if (f.cpos)
		arr->cpos = malloc(6  * sizeof(int));

	cube_to_arrays(cube, arr, f);

	return arr;
}

Cube
admissible_ep(Cube cube, PieceFilter f)
{
	CubeArray *arr = new_cubearray(cube, f);
	Cube ret;
	bool used[12] = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
	int i, j;

	for (i = 0; i < 12; i++)
		if (arr->ep[i] != -1)
			used[arr->ep[i]] = true;

	for (i = 0, j = 0; i < 12; i++) {
		for ( ; j < 11 && used[j]; j++);
		if (arr->ep[i] == -1)
			arr->ep[i] = j++;
	}

	ret = arrays_to_cube(arr, pf_ep);
	free_cubearray(arr, f);

	return ret;
}

Cube
compose(Cube c2, Cube c1)
{
	return compose_filtered(c2, c1, pf_all);
}

int
edge_slice(Edge e) {
	if (e < 0 || e > 11)
		return -1;

	if (e == FR || e == FL || e == BL || e == BR)
		return 0;
	if (e == UR || e == UL || e == DR || e == DL)
		return 1;

	return 2;
}

bool
equal(Cube c1, Cube c2)
{
	return c1.eofb  == c2.eofb  &&
	       c1.epose == c2.epose &&
	       c1.eposs == c2.eposs &&
	       c1.eposm == c2.eposm &&
	       c1.coud  == c2.coud  &&
	       c1.cp    == c2.cp    &&
	       c1.cpos  == c2.cpos;
}

Cube
inverse_cube(Cube cube)
{
	CubeArray inv;
	Cube ret;
	int i, ep[12];

	for (i = 0; i < 12; i++)
		ep[i] = where_is_edge(cube, i);
	inv = (CubeArray){.ep = ep};
	ret = arrays_to_cube(&inv, pf_ep);

	ret.eofb = ((int)eo_invtable_e[cube.eofb][cube.epose]) |
		   ((int)eo_invtable_m[cube.eofb][cube.eposm]) |
		   ((int)eo_invtable_s[cube.eofb][cube.eposs]);
	ret.eorl = ((int)eo_invtable_e[cube.eorl][cube.epose]) |
		   ((int)eo_invtable_m[cube.eorl][cube.eposm]) |
		   ((int)eo_invtable_s[cube.eorl][cube.eposs]);
	ret.eoud = ((int)eo_invtable_e[cube.eoud][cube.epose]) |
		   ((int)eo_invtable_m[cube.eoud][cube.eposm]) |
		   ((int)eo_invtable_s[cube.eoud][cube.eposs]);
	ret.cp   = cp_invtable[cube.cp];
	ret.cpos = cpos_invtable[cube.cpos];
	ret.coud = co_invtable[cube.coud][cube.cp];
	ret.corl = co_invtable[cube.corl][cube.cp];
	ret.cofb = co_invtable[cube.cofb][cube.cp];

	return ret;
}

bool
is_admissible(Cube cube) {

	/* TODO: this should check consistency of different orientations */
	/* check also that centers are opposite and admissible */

	CubeArray *a = new_cubearray(cube, pf_all);
	int parity;
	bool perm;

	perm   = is_perm(a->ep,  12)  &&
	         is_perm(a->cp,   8)  &&
	         is_perm(a->cpos, 6);
	parity = perm_sign(a->ep,  12) +
	         perm_sign(a->cp,   8) +
	         perm_sign(a->cpos, 6);

	free_cubearray(a, pf_all);

	return perm && parity % 2 == 0;
}

bool
is_solved(Cube cube)
{
	return equal(cube, (Cube){0});
}

bool
is_block_solved(Cube cube, Block block)
{
	int i;

	for (i = 0; i < 12; i++)
		if (block.edge[i] && !is_solved_edge(cube, i))
			return false;
	for (i = 0; i < 8; i++)
		if (block.corner[i] && !is_solved_corner(cube, i))
			return false;
	for (i = 0; i < 6; i++)
		if (block.center[i] && !is_solved_center(cube, i))
			return false;

	return true;
}

bool
is_solved_center(Cube cube, Center c)
{
	return what_center_at(cube, c) == c;
}

bool
is_solved_corner(Cube cube, Corner c)
{
	return what_corner_at(cube, c) == c &&
	       what_orientation_corner(cube.coud, c);
}

bool
is_solved_edge(Cube cube, Edge e)
{
	return what_edge_at(cube, e) == e &&
	       what_orientation_edge(cube.eofb, e);
}

int
piece_orientation(Cube cube, int piece, char *orientation)
{
	int arr[12], n, b, x;

	if (!strcmp(orientation, "eofb")) {
		x = cube.eofb;
		n = 12;
		b = 2;
	} else if (!strcmp(orientation, "eorl")) {
		x = cube.eorl;
		n = 12;
		b = 2;
	} else if (!strcmp(orientation, "eoud")) {
		x = cube.eoud;
		n = 12;
		b = 2;
	} else if (!strcmp(orientation, "coud")) {
		x = cube.coud;
		n = 8;
		b = 3;
	} else if (!strcmp(orientation, "corl")) {
		x = cube.corl;
		n = 8;
		b = 3;
	} else if (!strcmp(orientation, "cofb")) {
		x = cube.cofb;
		n = 8;
		b = 3;
	} else {
		return -1;
	}

	int_to_sum_zero_array(x, b, n, arr);
	if (piece < n)
		return arr[piece];

	return -1;
}

void
print_cube(Cube cube)
{
	static char edge_string[12][7] = {
		[UF] = "UF", [UL] = "UL", [UB] = "UB", [UR] = "UR",
		[DF] = "DF", [DL] = "DL", [DB] = "DB", [DR] = "DR",
		[FR] = "FR", [FL] = "FL", [BL] = "BL", [BR] = "BR"
	};

	static char corner_string[8][7] = {
		[UFR] = "UFR", [UFL] = "UFL", [UBL] = "UBL", [UBR] = "UBR",
		[DFR] = "DFR", [DFL] = "DFL", [DBL] = "DBL", [DBR] = "DBR"
	};

	static char center_string[6][7] = {
		[U_center] = "U", [D_center] = "D",
		[R_center] = "R", [L_center] = "L", 
		[F_center] = "F", [B_center] = "B"
	};

	for (int i = 0; i < 12; i++)
		printf(" %s ", edge_string[what_edge_at(cube, i)]);
	printf("\n");

	for (int i = 0; i < 12; i++)
		printf("  %d ", what_orientation_edge(cube.eofb, i));
	printf("\n");

	for (int i = 0; i < 8; i++)
		printf("%s ", corner_string[what_corner_at(cube, i)]);
	printf("\n");

	for (int i = 0; i < 8; i++)
		printf("  %d ", what_orientation_corner(cube.coud, i));
	printf("\n");

	for (int i = 0; i < 6; i++)
		printf("  %s ", center_string[what_center_at(cube, i)]);
	printf("\n");
}

Center
what_center_at(Cube cube, Center c)
{
	static bool initialized = false;
	static Center aux[FACTORIAL6][6];
	static int i;
	static unsigned int ui;
	static CubeArray *arr;

	if (!initialized) {
		for (ui = 0; ui < FACTORIAL6; ui++) {
			arr = new_cubearray((Cube){.cpos = ui}, pf_cpos);
			for (i = 0; i < 6; i++)
				aux[ui][i] = arr->cpos[i];
			free_cubearray(arr, pf_cpos);
		}

		initialized = true;
	}

	return aux[cube.cpos][c];
}

Corner
what_corner_at(Cube cube, Corner c)
{
	int i;
	unsigned int ui;
	CubeArray *arr;

	static bool initialized = false;
	static Corner aux[FACTORIAL8][8];

	if (!initialized) {
		for (ui = 0; ui < FACTORIAL8; ui++) {
			arr = new_cubearray((Cube){.cp = ui}, pf_cp);
			for (i = 0; i < 8; i++)
				aux[ui][i] = arr->cp[i];
			free_cubearray(arr, pf_cp);
		}

		initialized = true;
	}

	return aux[cube.cp][c];
}

Edge
what_edge_at(Cube cube, Edge e)
{
	Edge ret;
	CubeArray *arr = new_cubearray(cube, pf_ep);

	ret = arr->ep[e];

	free_cubearray(arr, pf_ep);
	return ret;
}

int
what_orientation_corner(int co, Corner c)
{
	static bool initialized = false;
	static int auxlast[POW3TO7];
	static int auxarr[8];
	static unsigned int ui;

	if (!initialized) {
		for (ui = 0; ui < POW3TO7; ui++) {
			int_to_sum_zero_array(ui, 3, 8, auxarr);
			auxlast[ui] = auxarr[7];
		}

		initialized = true;
	}

	if (c < 7)
		return (co / powint(3, c)) % 3;
	else
		return auxlast[co];
}

int
what_orientation_edge(int eo, Edge e)
{
	static bool initialized = false;
	static int auxlast[POW2TO11];
	static int auxarr[12];
	static unsigned int ui;

	if (!initialized) {
		for (ui = 0; ui < POW2TO11; ui++) {
			int_to_sum_zero_array(ui, 2, 12, auxarr);
			auxlast[ui] = auxarr[11];
		}

		initialized = true;
	}

	if (e < 11)
		return (eo & (1 << e)) ? 1 : 0;
	else
		return auxlast[eo];
}

Center
where_is_center(Cube cube, Center c)
{
	static bool initialized = false;
	static Center aux[FACTORIAL6][6];
	static int i;
	static unsigned int ui;
	static CubeArray *arr;

	if (!initialized) {
		for (ui = 0; ui < FACTORIAL6; ui++) {
			arr = new_cubearray((Cube){.cpos = ui}, pf_cpos);
			for (i = 0; i < 6; i++)
				aux[ui][arr->cpos[i]] = i;
			free_cubearray(arr, pf_cpos);
		}

		initialized = true;
	}

	return aux[cube.cpos][c];
}

Corner
where_is_corner(Cube cube, Corner c)
{
	static bool initialized = false;
	static Corner aux[FACTORIAL8][8];
	static int i;
	static unsigned int ui;
	static CubeArray *arr;

	if (!initialized) {
		for (ui = 0; ui < FACTORIAL8; ui++) {
			arr = new_cubearray((Cube){.cp = ui}, pf_cp);
			for (i = 0; i < 8; i++)
				aux[ui][arr->cp[i]] = i;
			free_cubearray(arr, pf_cp);
		}

		initialized = true;
	}
	return aux[cube.cp][c];
}

Edge
where_is_edge(Cube c, Edge e)
{
	int r0, r1, r2;

	static bool initialized = false;
	static int aux[3][BINOM12ON4*FACTORIAL4][12];
	static int i, j;
	static unsigned int ui;
	static CubeArray *arr;

	if (!initialized) {
		for (ui = 0; ui < BINOM12ON4*FACTORIAL4; ui++) {
			for (i = 0; i < 3; i++)
				for (j = 0; j < 12; j++)
					aux[i][ui][j] = -1;
				
			arr = new_cubearray((Cube){.epose = ui}, pf_e);
			for (i = 0; i < 12; i++)
				if (edge_slice(arr->ep[i]) == 0)
					aux[0][ui][arr->ep[i]] = i;
			free_cubearray(arr, pf_e);

			arr = new_cubearray((Cube){.eposs = ui}, pf_s);
			for (i = 0; i < 12; i++)
				if (edge_slice(arr->ep[i]) == 1)
					aux[1][ui][arr->ep[i]] = i;
			free_cubearray(arr, pf_s);

			arr = new_cubearray((Cube){.eposm = ui}, pf_m);
			for (i = 0; i < 12; i++)
				if (edge_slice(arr->ep[i]) == 2)
					aux[2][ui][arr->ep[i]] = i;
			free_cubearray(arr, pf_m);
		}

		initialized = true;
	}

	r0 = aux[0][c.epose][e];
	r1 = aux[1][c.eposs][e];
	r2 = aux[2][c.eposm][e];
	return MAX(r0, MAX(r1, r2));
}

static bool
read_invtables_file(void)
{
	init_env();

	FILE *f;
	char fname[strlen(tabledir)+20];
	int b;
	unsigned int ui, meeo, meco, mecp, mecpos;
	bool r;

	strcpy(fname, tabledir);
	strcat(fname, "/invtables");

	if ((f = fopen(fname, "rb")) == NULL)
		return false;

	b = sizeof(uint16_t);
	r = true;
	meeo = BINOM12ON4*FACTORIAL4;
	meco = FACTORIAL8;
	mecp = FACTORIAL8;
	mecpos = FACTORIAL6;

	for (ui = 0; ui < POW2TO11; ui++) {
		r = r && fread(eo_invtable_e[ui], b, meeo, f) == meeo;
		r = r && fread(eo_invtable_m[ui], b, meeo, f) == meeo;
		r = r && fread(eo_invtable_s[ui], b, meeo, f) == meeo;
	}

	for (ui = 0; ui < POW3TO7; ui++) {
		r = r && fread(co_invtable[ui], b, meco, f) == meco;
	}

	r = r && fread(cp_invtable,   b, mecp,   f) == mecp;
	r = r && fread(cpos_invtable, b, mecpos, f) == mecpos;

	fclose(f);
	return r;
}

static bool
write_invtables_file(void)
{
	init_env();

	FILE *f;
	char fname[strlen(tabledir)+20];
	unsigned int ui, meeo, meco, mecp, mecpos;
	int b;
	bool r;

	strcpy(fname, tabledir);
	strcat(fname, "/invtables");

	if ((f = fopen(fname, "wb")) == NULL)
		return false;

	b = sizeof(uint16_t);
	r = true;
	meeo = BINOM12ON4*FACTORIAL4;
	meco = FACTORIAL8;
	mecp = FACTORIAL8;
	mecpos = FACTORIAL6;

	for (ui = 0; ui < POW2TO11; ui++) {
		r = r && fwrite(eo_invtable_e[ui], b, meeo, f) == meeo;
		r = r && fwrite(eo_invtable_m[ui], b, meeo, f) == meeo;
		r = r && fwrite(eo_invtable_s[ui], b, meeo, f) == meeo;
	}

	for (ui = 0; ui < POW3TO7; ui++) {
		r = r && fwrite(co_invtable[ui], b, meco, f) == meco;
	}

	r = r && fwrite(cp_invtable,   b, mecp,   f) == mecp;
	r = r && fwrite(cpos_invtable, b, mecpos, f) == mecpos;

	fclose(f);
	return r;
}

void
init_inverse(void)
{
	static bool initialized = false;
	if (initialized)
		return;
	initialized = true;

	if (read_invtables_file())
		return;

	fprintf(stderr, "Cannot load invtables, generating it\n");

	CubeArray *aux, *inv;
	Cube c;
	int i, j, eoaux[12], eoinv[12];
	unsigned int ui, uj;

	aux = new_cubearray((Cube){0}, pf_all);
	inv = new_cubearray((Cube){0}, pf_all);

	for (ui = 0; ui < POW2TO11; ui++) {
		int_to_sum_zero_array(ui, 2, 12, eoaux);
		for (uj = 0; uj < BINOM12ON4*FACTORIAL4; uj++) {
			for (j = 0; j < 12; j++)
				eoinv[j] = 0;
			c = (Cube){.epose = uj, .eposm = 0, .eposs = 0};
			eoinv[FR] = eoaux[where_is_edge(c, FR)];
			eoinv[FL] = eoaux[where_is_edge(c, FL)];
			eoinv[BL] = eoaux[where_is_edge(c, BL)];
			eoinv[BR] = eoaux[where_is_edge(c, BR)];
			eo_invtable_e[ui][uj] = digit_array_to_int(eoinv,11,2);
 
			for (j = 0; j < 12; j++)
				eoinv[j] = 0;
			c = (Cube){.epose = 0, .eposm = uj, .eposs = 0};
			eoinv[UF] = eoaux[where_is_edge(c, UF)];
			eoinv[UB] = eoaux[where_is_edge(c, UB)];
			eoinv[DF] = eoaux[where_is_edge(c, DF)];
			eoinv[DB] = eoaux[where_is_edge(c, DB)];
			eo_invtable_m[ui][uj] = digit_array_to_int(eoinv,11,2);
 
			for (j = 0; j < 12; j++)
				eoinv[j] = 0;
			c = (Cube){.epose = 0, .eposm = 0, .eposs = uj};
			eoinv[UL] = eoaux[where_is_edge(c, UL)];
			eoinv[UR] = eoaux[where_is_edge(c, UR)];
			eoinv[DL] = eoaux[where_is_edge(c, DL)];
			eoinv[DR] = eoaux[where_is_edge(c, DR)];
			eo_invtable_s[ui][uj] = digit_array_to_int(eoinv,11,2);
		}
	}

	for (ui = 0; ui < FACTORIAL8; ui++) {
		cube_to_arrays((Cube){.cp = ui}, aux, pf_cp);
		for (i = 0; i < 8; i++)
			inv->cp[aux->cp[i]] = i;
		cp_invtable[ui] = (uint16_t)arrays_to_cube(inv, pf_cp).cp;

		for (uj = 0; uj < POW3TO7; uj++) {
			cube_to_arrays((Cube){.coud = uj}, aux, pf_coud);
			for (i = 0; i < 8; i++)
				inv->coud[aux->cp[i]] = (3-aux->coud[i])%3;
			co_invtable[uj][ui] =
			    (uint16_t)arrays_to_cube(inv, pf_coud).coud;
		}
	}
	
	for (ui = 0; ui < FACTORIAL6; ui++) {
		cube_to_arrays((Cube){.cpos = ui}, aux, pf_cpos);
		for (i = 0; i < 6; i++)
			inv->cpos[aux->cpos[i]] = i;
		cpos_invtable[ui] =
		    (uint16_t)arrays_to_cube(inv, pf_cpos).cpos;
	}

	free_cubearray(aux, pf_all);
	free_cubearray(inv, pf_all);

	if (!write_invtables_file())
		fprintf(stderr, "Error writing invtables\n");
}

void
init_cube(void)
{
	init_inverse();
}
