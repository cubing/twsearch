#include "symcoord.h"

/* These constants have been computed generating the respective SymData */
#define CLASSES_CP_16        2768
#define CLASSES_EOFBEPOS_16  64430

static uint64_t    index_cp_sym16(Cube cube);
static uint64_t    index_eofbepos_sym16(Cube cube);
static uint64_t    index_drud_sym16(Cube cube);
static uint64_t    index_drudfin_noE_sym16(Cube cube);
static uint64_t    index_nxopt31(Cube cube);

static uint64_t    move_cp_sym16(Move m, uint64_t ind);
static uint64_t    move_eofbepos_sym16(Move m, uint64_t ind);
static uint64_t    move_drud_sym16(Move m, uint64_t ind);
static uint64_t    move_drudfin_noE_sym16(Move m, uint64_t ind);
static uint64_t    move_nxopt31(Move m, uint64_t ind);

static int         tfind_from_mask(uint64_t mask, Trans *ret);
static int         tfind_drud_sym16(uint64_t ind, Trans *ret);
static int         tfind_drudfin_noE_sym16(uint64_t ind, Trans *ret);
static int         tfind_nxopt31(uint64_t ind, Trans *ret);

static uint64_t    transform_cp(Trans t, uint64_t ind);
static uint64_t    transform_eofbepos(Trans t, uint64_t ind);
static uint64_t    transform_drud_sym16(Trans t, uint64_t ind);
static uint64_t    transform_drudfin_noE_sym16(Trans t, uint64_t ind);
static uint64_t    transform_nxopt31(Trans t, uint64_t ind);

static void        gensym(SymData *sd);
static void        init_symc_moves(void);
static void        init_symc_trans(void);
static bool        read_symdata_file(SymData *sd);
static bool        read_symc_moves_file(void);
static bool        read_symc_trans_file(void);
static bool        write_symdata_file(SymData *sd);
static bool        write_symc_moves_file(void);
static bool        write_symc_trans_file(void);

/* Some tables ***************************************************************/

static uint64_t    move_cp_16[NMOVES][CLASSES_CP_16];
static uint64_t    move_eofbepos_16[NMOVES][CLASSES_EOFBEPOS_16];

static int         trans_eofbepos[NTRANS][POW2TO11*BINOM12ON4];
static int         trans_epud[NTRANS][FACTORIAL8];
static int         trans_cpud_separate[NTRANS][BINOM8ON4];

static Trans       ttrep_move_cp_16[NMOVES][CLASSES_CP_16];
static Trans       ttrep_move_eofbepos_16[NMOVES][CLASSES_EOFBEPOS_16];


/* Transformation groups and symmetry data ***********************************/

static Trans
trans_group_udfix[16] = {
	uf, ur, ub, ul,
	df, dr, db, dl,
	uf_mirror, ur_mirror, ub_mirror, ul_mirror,
	df_mirror, dr_mirror, db_mirror, dl_mirror,
};

static SymData
sd_cp_16 = {
	.filename  = "sd_cp_16_new",
	.coord     = &coord_cp,
	.sym_coord = &coord_cp_sym16,
	.ntrans    = 16,
	.trans     = trans_group_udfix,
	.transform = transform_cp,
};

static SymData
sd_eofbepos_16 = {
	.filename  = "sd_eofbepos_16_new",
	.coord     = &coord_eofbepos,
	.sym_coord = &coord_eofbepos_sym16,
	.ntrans    = 16,
	.trans     = trans_group_udfix,
	.transform = transform_eofbepos,
};

SymData * all_sd[] = {
	&sd_cp_16,
	&sd_eofbepos_16,
	NULL
};


/* Coordinates and their implementation **************************************/

Coordinate
coord_eofbepos_sym16 = {
	.index     = index_eofbepos_sym16,
	.move      = move_eofbepos_sym16,
};

Coordinate
coord_cp_sym16 = {
	.index     = index_cp_sym16,
	.move      = move_cp_sym16,
};

Coordinate
coord_drud_sym16 = {
	.index     = index_drud_sym16,
	.move      = move_drud_sym16,
	.max       = POW3TO7 * CLASSES_EOFBEPOS_16,
	.base      = &coord_eofbepos_sym16,
	.transform = transform_drud_sym16,
	.tfind     = tfind_drud_sym16,
};

Coordinate
coord_drudfin_noE_sym16 = {
	.index     = index_drudfin_noE_sym16,
	.move      = move_drudfin_noE_sym16,
	.max       = FACTORIAL8 * CLASSES_CP_16,
	.base      = &coord_cp_sym16,
	.transform = transform_drudfin_noE_sym16,
	.tfind     = tfind_drudfin_noE_sym16,
};

Coordinate
coord_nxopt31 = {
	.index     = index_nxopt31,
	.move      = move_nxopt31,
	.max       = POW3TO7 * BINOM8ON4 * CLASSES_EOFBEPOS_16,
	.base      = &coord_eofbepos_sym16,
	.transform = transform_nxopt31,
	.tfind     = tfind_nxopt31,
};

/* Functions *****************************************************************/

static uint64_t
index_cp_sym16(Cube cube)
{
	return sd_cp_16.class[coord_cp.index(cube)];
}

static uint64_t
index_drud_sym16(Cube cube)
{
	Trans t;

	t = sd_eofbepos_16.transtorep[coord_eofbepos.index(cube)];

	return index_eofbepos_sym16(cube) * POW3TO7 + co_ttable[t][cube.coud];
}

static uint64_t
index_drudfin_noE_sym16(Cube cube)
{
	Trans t;
	Cube c;

	t = sd_cp_16.transtorep[coord_cp.index(cube)];
	c = apply_trans(t, cube);

	/* TODO: add transform to coord_epud to make this faster */
	return index_cp_sym16(c) * FACTORIAL8 + coord_epud.index(c);
}

static uint64_t
index_eofbepos_sym16(Cube cube)
{
	return sd_eofbepos_16.class[coord_eofbepos.index(cube)];
}

static uint64_t
index_nxopt31(Cube cube)
{
	Trans t;
	uint64_t a;
	int coud, cp;
	
	t = sd_eofbepos_16.transtorep[coord_eofbepos.index(cube)];
	coud = co_ttable[t][cube.coud];
	cp   = cp_ttable[t][cube.cp];
	a = (index_eofbepos_sym16(cube)*POW3TO7) + coud;

	return a * BINOM8ON4 + coord_cpud_separate.index((Cube){.cp = cp});
}

static uint64_t
move_cp_sym16(Move m, uint64_t ind)
{
	return move_cp_16[m][ind];
}

static uint64_t
move_eofbepos_sym16(Move m, uint64_t ind)
{
	return move_eofbepos_16[m][ind];
}

static uint64_t
move_drud_sym16(Move m, uint64_t ind)
{
	uint64_t coud, eofbepos;
	Trans ttr;

	eofbepos = move_eofbepos_16[m][ind / POW3TO7];
	ttr = ttrep_move_eofbepos_16[m][ind / POW3TO7];
	coud = coud_mtable[m][ind % POW3TO7];
	coud = co_ttable[ttr][coud]; /* Source is always coud */

	return eofbepos * POW3TO7 + coud;
}

static uint64_t
move_drudfin_noE_sym16(Move m, uint64_t ind)
{
	uint64_t cp, epud;
	Trans ttr;

	cp = move_cp_16[m][ind / FACTORIAL8];
	ttr = ttrep_move_cp_16[m][ind / FACTORIAL8];
	epud = coord_epud.move(m, ind % FACTORIAL8);
	epud = trans_epud[ttr][epud];

	return cp * FACTORIAL8 + epud;
}

static uint64_t
move_nxopt31(Move m, uint64_t ind)
{
	uint64_t eofbepos, cpsep, coud;
	Trans ttr;

	eofbepos = ind / (POW3TO7 * BINOM8ON4);
	coud = (ind / BINOM8ON4) % POW3TO7;
	cpsep = ind % BINOM8ON4;

	ttr = ttrep_move_eofbepos_16[m][eofbepos];
	eofbepos = move_eofbepos_16[m][eofbepos];
	coud = coud_mtable[m][coud];
	coud = co_ttable[ttr][coud]; /* Source is always coud */
	cpsep = coord_cpud_separate.move(m, cpsep);
	cpsep = trans_cpud_separate[ttr][cpsep];

	return (eofbepos * POW3TO7 + coud) * BINOM8ON4 + cpsep;
}

static uint64_t
transform_cp(Trans t, uint64_t ind)
{
	return cp_ttable[t][ind];
}

static uint64_t
transform_eofbepos(Trans t, uint64_t ind)
{
	return trans_eofbepos[t][ind];
}

static uint64_t
transform_drud_sym16(Trans t, uint64_t ind)
{
	uint64_t coud, eofbepos;

	eofbepos = ind / POW3TO7; /* Assum trans fixes eofbepos */
	coud = co_ttable[t][ind % POW3TO7]; /* Source is always coud */

	return eofbepos * POW3TO7 + coud;
}

static uint64_t
transform_drudfin_noE_sym16(Trans t, uint64_t ind)
{
	uint64_t cp, epud;

	cp = ind / FACTORIAL8; /* Assume trans fixes cp */
	epud = trans_epud[t][ind % FACTORIAL8];

	return cp * FACTORIAL8 + epud;
}

static uint64_t
transform_nxopt31(Trans t, uint64_t ind)
{
	uint64_t eofbepos, cpsep, coud;

	eofbepos = ind / (POW3TO7 * BINOM8ON4);
	coud = (ind / BINOM8ON4) % POW3TO7;
	cpsep = ind % BINOM8ON4;

	coud = co_ttable[t][coud]; /* Source is always coud */
	cpsep = trans_cpud_separate[t][cpsep];

	return (eofbepos * POW3TO7 + coud) * BINOM8ON4 + cpsep;
}

static int
tfind_from_mask(uint64_t mask, Trans *ret)
{
	Trans t;
	int i = 0;

	for (t = uf; t < NTRANS; t++)
		if (((uint64_t)1 << t) & mask)
			ret[i++] = t;

	return i;
}

static int
tfind_drud_sym16(uint64_t ind, Trans *ret)
{
	uint64_t mask = sd_eofbepos_16.selfsim[ind / POW3TO7];

	return tfind_from_mask(mask, ret);
}

static int
tfind_drudfin_noE_sym16(uint64_t ind, Trans *ret)
{
	uint64_t mask = sd_cp_16.selfsim[ind / FACTORIAL8];

	return tfind_from_mask(mask, ret);
}

static int
tfind_nxopt31(uint64_t ind, Trans *ret)
{
	uint64_t mask = sd_eofbepos_16.selfsim[ind / (POW3TO7 * BINOM8ON4)];

	return tfind_from_mask(mask, ret);
}

/* Other functions ***********************************************************/

void
free_sd(SymData *sd)
{
	if (sd->generated) {
		free(sd->class);
		free(sd->unsym);
		free(sd->transtorep);
	}

	sd->generated = false;
}

static void
gensym(SymData *sd)
{
	uint64_t i, in, nreps = 0;
	Trans t;
	int j;

	if (sd->generated)
		return;

	sd->class      = malloc(sd->coord->max * sizeof(uint64_t));
	sd->unsym      = malloc(sd->coord->max * sizeof(uint64_t));
	sd->transtorep = malloc(sd->coord->max * sizeof(Trans));
	sd->selfsim    = malloc(sd->coord->max * sizeof(uint64_t));

	if (read_symdata_file(sd)) {
		sd->generated = true;
		return;
	}

	fprintf(stderr, "Cannot load %s, generating it\n", sd->filename);

	for (i = 0; i < sd->coord->max; i++)
		sd->class[i] = sd->coord->max + 1;

	for (i = 0; i < sd->coord->max; i++) {
		if (sd->class[i] == sd->coord->max + 1) {
			sd->unsym[nreps] = i;
			sd->transtorep[i] = uf;
			sd->selfsim[nreps] = (uint64_t)0;
			for (j = 0; j < sd->ntrans; j++) {
				t = sd->trans[j];
				in = sd->transform(t, i);
				sd->class[in] = nreps;
				if (in == i)
					sd->selfsim[nreps] |=
					    ((uint64_t)1 << t);
				else
					sd->transtorep[in] = inverse_trans(t);
			}
			nreps++;
		}
	}

	sd->sym_coord->max = nreps;
	sd->unsym          = realloc(sd->unsym,   nreps * sizeof(uint64_t));
	sd->selfsim        = realloc(sd->selfsim, nreps * sizeof(uint64_t));
	sd->generated      = true;

	fprintf(stderr, "Found %" PRIu64 " classes\n", nreps);

	if (!write_symdata_file(sd))
		fprintf(stderr, "Error writing SymData file\n");

	return;
}

static bool
read_symdata_file(SymData *sd)
{
	init_env();

	FILE *f;
	char fname[strlen(tabledir)+100];
	uint64_t n = sd->coord->max, *sn = &sd->sym_coord->max;
	bool r = true;

	strcpy(fname, tabledir);
	strcat(fname, "/");
	strcat(fname, sd->filename);

	if ((f = fopen(fname, "rb")) == NULL)
		return false;

	r = r && fread(&sd->sym_coord->max, sizeof(uint64_t), 1,   f) == 1;
	r = r && fread(sd->unsym,           sizeof(uint64_t), *sn, f) == *sn;
	r = r && fread(sd->selfsim,         sizeof(uint64_t), *sn, f) == *sn;
	r = r && fread(sd->class,           sizeof(uint64_t), n,   f) == n;
	r = r && fread(sd->transtorep,      sizeof(Trans),    n,   f) == n;

	fclose(f);
	return r;
}

static bool
read_symc_moves_file(void)
{
	init_env();

	Move m;
	bool r = true;
	FILE *f;
	char fname[strlen(tabledir)+100];

	strcpy(fname, tabledir);
	strcat(fname, "/symc_moves");

	if ((f = fopen(fname, "rb")) == NULL)
		return false;

	for (m = 0; m < NMOVES; m++) {
		r = r && fread(move_cp_16[m], sizeof(uint64_t),
		    CLASSES_CP_16, f) == CLASSES_CP_16;
		r = r && fread(move_eofbepos_16[m], sizeof(uint64_t),
		    CLASSES_EOFBEPOS_16, f) == CLASSES_EOFBEPOS_16;

		r = r && fread(ttrep_move_cp_16[m], sizeof(Trans),
		    CLASSES_CP_16, f) == CLASSES_CP_16;
		r = r && fread(ttrep_move_eofbepos_16[m], sizeof(Trans),
		    CLASSES_EOFBEPOS_16, f) == CLASSES_EOFBEPOS_16;
	}

	fclose(f);
	return r;
}

static bool
read_symc_trans_file(void)
{
	init_env();

	Trans t;
	bool r = true;
	FILE *f;
	char fname[strlen(tabledir)+100];

	strcpy(fname, tabledir);
	strcat(fname, "/symc_trans");

	if ((f = fopen(fname, "rb")) == NULL)
		return false;

	for (t = 0; t < NTRANS; t++) {
		r = r && fread(trans_eofbepos[t], sizeof(int),
		    POW2TO11*BINOM12ON4, f) == POW2TO11*BINOM12ON4;
		r = r && fread(trans_epud[t], sizeof(int),
		    FACTORIAL8, f) == FACTORIAL8;
		r = r && fread(trans_cpud_separate[t], sizeof(int),
		    BINOM8ON4, f) == BINOM8ON4;
	}

	fclose(f);
	return r;
}

static bool
write_symdata_file(SymData *sd)
{
	init_env();

	FILE *f;
	char fname[strlen(tabledir)+100];
	uint64_t n = sd->coord->max, *sn = &sd->sym_coord->max;
	bool r = true;

	strcpy(fname, tabledir);
	strcat(fname, "/");
	strcat(fname, sd->filename);

	if ((f = fopen(fname, "wb")) == NULL)
		return false;

	r = r && fwrite(&sd->sym_coord->max, sizeof(uint64_t), 1,   f) == 1;
	r = r && fwrite(sd->unsym,           sizeof(uint64_t), *sn, f) == *sn;
	r = r && fwrite(sd->selfsim,         sizeof(uint64_t), *sn, f) == *sn;
	r = r && fwrite(sd->class,           sizeof(uint64_t), n,   f) == n;
	r = r && fwrite(sd->transtorep,      sizeof(Trans),    n,   f) == n;

	fclose(f);
	return r;
}

static bool
write_symc_moves_file(void)
{
	init_env();

	Move m;
	bool r = true;
	FILE *f;
	char fname[strlen(tabledir)+100];

	strcpy(fname, tabledir);
	strcat(fname, "/symc_moves");

	if ((f = fopen(fname, "wb")) == NULL)
		return false;

	for (m = 0; m < NMOVES; m++) {
		r = r && fwrite(move_cp_16[m], sizeof(uint64_t),
		    CLASSES_CP_16, f) == CLASSES_CP_16;
		r = r && fwrite(move_eofbepos_16[m], sizeof(uint64_t),
		    CLASSES_EOFBEPOS_16, f) == CLASSES_EOFBEPOS_16;

		r = r && fwrite(ttrep_move_cp_16[m], sizeof(Trans),
		    CLASSES_CP_16, f) == CLASSES_CP_16;
		r = r && fwrite(ttrep_move_eofbepos_16[m], sizeof(Trans),
		    CLASSES_EOFBEPOS_16, f) == CLASSES_EOFBEPOS_16;
	}

	fclose(f);
	return r;
}

static bool
write_symc_trans_file(void)
{
	init_env();

	Trans t;
	bool r = true;
	FILE *f;
	char fname[strlen(tabledir)+100];

	strcpy(fname, tabledir);
	strcat(fname, "/symc_trans");

	if ((f = fopen(fname, "wb")) == NULL)
		return false;

	for (t = 0; t < NTRANS; t++) {
		r = r && fwrite(trans_eofbepos[t], sizeof(int),
		    POW2TO11*BINOM12ON4, f) == POW2TO11*BINOM12ON4;
		r = r && fwrite(trans_epud[t], sizeof(int),
		    FACTORIAL8, f) == FACTORIAL8;
		r = r && fwrite(trans_cpud_separate[t], sizeof(int),
		    BINOM8ON4, f) == BINOM8ON4;
	}

	fclose(f);
	return r;
}

static void
init_symc_moves(void)
{
	uint64_t i, ii, coo;
	Move j;

	if (read_symc_moves_file())
		return;

	for (i = 0; i < CLASSES_CP_16; i++) {
		ii = sd_cp_16.unsym[i];
		for (j = 0; j < NMOVES; j++) {
			coo = sd_cp_16.coord->move(j, ii);
			move_cp_16[j][i] = sd_cp_16.class[coo];
			ttrep_move_cp_16[j][i] = sd_cp_16.transtorep[coo];
		}
	}

	for (i = 0; i < CLASSES_EOFBEPOS_16; i++) {
		ii = sd_eofbepos_16.unsym[i];
		for (j = 0; j < NMOVES; j++) {
			coo = sd_eofbepos_16.coord->move(j, ii);
			move_eofbepos_16[j][i] = sd_eofbepos_16.class[coo];
			ttrep_move_eofbepos_16[j][i] =
			    sd_eofbepos_16.transtorep[coo];
		}
	}

	if (!write_symc_moves_file())
		fprintf(stderr, "Error writing SymMoves file\n");
}

void
init_symc_trans(void)
{
	uint64_t i;
	int j, cp;
	int epe[4] = {FR, FL, BL, BR};
	int a[12] = { [8] = 8, [9] = 9, [10] = 10, [11] = 11 };
	Cube c;
	CubeArray *arr, *aux;
	Trans t;

	if (read_symc_trans_file())
		return;

	for (i = 0; i < POW2TO11*BINOM12ON4; i++) {
		for (j = 0; j < 16; j++) {
			t = trans_group_udfix[j];

			arr = new_cubearray((Cube){0}, pf_edges);
			int_to_sum_zero_array(i % POW2TO11, 2, 12, arr->eofb);
			epos_to_compatible_ep((i / POW2TO11)*24, arr->ep, epe);
			fix_eorleoud(arr);
			c = arrays_to_cube(arr, pf_edges);
			free_cubearray(arr, pf_edges);

			c = apply_trans(t, c);
			trans_eofbepos[t][i] = (c.epose/24)*POW2TO11 + c.eofb;
		}
	}

	aux = malloc(sizeof(CubeArray));
	aux->ep = a;
	for (i = 0; i < FACTORIAL8; i++) {
		index_to_perm(i, 8, a);
		c = arrays_to_cube(aux, pf_ep);
		for (j = 0; j < 16; j++) {
			t = trans_group_udfix[j];
			arr = new_cubearray(apply_trans(t, c), pf_ep);
			trans_epud[t][i] = perm_to_index(arr->ep, 8);
			free_cubearray(arr, pf_ep);
		}
	}
	free(aux);

	for (i = 0; i < BINOM8ON4; i++) {
		cp = cpud_separate_ant[i];
		for (j = 0; j < 16; j++) {
			t = trans_group_udfix[j];
			trans_cpud_separate[t][i] =
			    cpud_separate_ind[cp_ttable[t][cp]];
		}
	}

	if (!write_symc_trans_file())
		fprintf(stderr, "Error writing SymTrans file\n");
}

void
init_symcoord(void)
{
	int i;

	static bool initialized = false;
	if (initialized)
		return;
	initialized = true;

	init_coord();

	init_symc_trans();

	for (i = 0; all_sd[i] != NULL; i++)
		gensym(all_sd[i]);

	init_symc_moves();
}

