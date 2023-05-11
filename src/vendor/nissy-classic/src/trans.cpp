#include "trans.h"

/* Local functions ***********************************************************/

static bool        read_ttables_file(void);
static Cube        rotate_via_compose(Trans r, Cube c, PieceFilter f);
static bool        write_ttables_file(void);

/* Tables and other data *****************************************************/

static int              ep_mirror[12] = {
	[UF] = UF, [UL] = UR, [UB] = UB, [UR] = UL,
	[DF] = DF, [DL] = DR, [DB] = DB, [DR] = DL,
	[FR] = FL, [FL] = FR, [BL] = BR, [BR] = BL
};

static int              cp_mirror[8] = {
	[UFR] = UFL, [UFL] = UFR, [UBL] = UBR, [UBR] = UBL,
	[DFR] = DFL, [DFL] = DFR, [DBL] = DBR, [DBR] = DBL
};

static int              cpos_mirror[6] = {
	[U_center] = U_center, [D_center] = D_center,
	[R_center] = L_center, [L_center] = R_center,
	[F_center] = F_center, [B_center] = B_center
};

static char             rotation_alg_string[100][NROTATIONS] = {
	[uf] = "",     [ur] = "y",    [ub] = "y2",    [ul] = "y3",
	[df] = "z2",   [dr] = "y z2", [db] = "x2",    [dl] = "y3 z2",
	[rf] = "z3",   [rd] = "z3 y", [rb] = "z3 y2", [ru] = "z3 y3",
	[lf] = "z",    [ld] = "z y3", [lb] = "z y2",  [lu] = "z y",
	[fu] = "x y2", [fr] = "x y",  [fd] = "x",     [fl] = "x y3",
	[bu] = "x3",   [br] = "x3 y", [bd] = "x3 y2", [bl] = "x3 y3",
};

static int              epose_source[NTRANS];  /* 0=epose, 1=eposs, 2=eposm */
static int              eposs_source[NTRANS];
static int              eposm_source[NTRANS];
static int              eofb_source[NTRANS];   /* 0=eoud,  1=eorl,  2=eofb  */
static int              eorl_source[NTRANS];
static int              eoud_source[NTRANS];
static int              coud_source[NTRANS];   /* 0=coud,  1=corl,  2=cofb  */
static int              cofb_source[NTRANS];
static int              corl_source[NTRANS];

int                     epose_ttable[NTRANS][FACTORIAL12/FACTORIAL8];
int                     eposs_ttable[NTRANS][FACTORIAL12/FACTORIAL8];
int                     eposm_ttable[NTRANS][FACTORIAL12/FACTORIAL8];
int                     eo_ttable[NTRANS][POW2TO11];
int                     cp_ttable[NTRANS][FACTORIAL8];
int                     co_ttable[NTRANS][POW3TO7];
int                     cpos_ttable[NTRANS][FACTORIAL6];
Move                    moves_ttable[NTRANS][NMOVES];

/* Local functions implementation ********************************************/

static bool
read_ttables_file(void)
{
	init_env();

	FILE *f;
	char fname[strlen(tabledir)+20];
	int b = sizeof(int);
	bool r = true;
	Move m;

	/* Table sizes, used for reading and writing files */
	uint64_t me[11] = {
		[0] = FACTORIAL12/FACTORIAL8,
		[1] = FACTORIAL12/FACTORIAL8,
		[2] = FACTORIAL12/FACTORIAL8,
		[3] = POW2TO11,
		[4] = FACTORIAL8,
		[5] = POW3TO7,
		[6] = FACTORIAL6,
		[7] = NMOVES
	};

	strcpy(fname, tabledir);
	strcat(fname, "/");
	strcat(fname, "ttables");

	if ((f = fopen(fname, "rb")) == NULL)
		return false;

	for (m = 0; m < NTRANS; m++) {
		r = r && fread(epose_ttable[m], b, me[0], f) == me[0];
		r = r && fread(eposs_ttable[m], b, me[1], f) == me[1];
		r = r && fread(eposm_ttable[m], b, me[2], f) == me[2];
		r = r && fread(eo_ttable[m],    b, me[3], f) == me[3];
		r = r && fread(cp_ttable[m],    b, me[4], f) == me[4];
		r = r && fread(co_ttable[m],    b, me[5], f) == me[5];
		r = r && fread(cpos_ttable[m],  b, me[6], f) == me[6];
		r = r && fread(moves_ttable[m], b, me[7], f) == me[7];
	}

	fclose(f);
	return r;
}

static Cube
rotate_via_compose(Trans r, Cube c, PieceFilter f)
{
	static int zero12[12] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };
	static int zero8[8]   = { 0, 0, 0, 0, 0, 0, 0, 0 };
	static CubeArray ma = {
		.ep   = ep_mirror,
		.eofb = zero12,
		.eorl = zero12,
		.eoud = zero12,
		.cp   = cp_mirror,
		.coud = zero8,
		.corl = zero8,
		.cofb = zero8,
		.cpos = cpos_mirror
	};

	Alg *inv = inverse_alg(rotation_alg(r));
	Cube ret = {0};

	if (r >= NROTATIONS)
		ret = move_via_arrays(&ma, ret, f);
	ret = apply_alg_generic(inv, ret, f, true);

	ret = compose_filtered(c, ret, f);

	ret = apply_alg_generic(rotation_alg(r), ret, f, true);
	if (r >= NROTATIONS)
		ret = move_via_arrays(&ma, ret, f);

	free_alg(inv);
	return ret;
}

static bool
write_ttables_file(void)
{
	init_env();

	FILE *f;
	char fname[strlen(tabledir)+20];
	bool r = true;
	int b = sizeof(int);
	Move m;

	/* Table sizes, used for reading and writing files */
	uint64_t me[11] = {
		[0] = FACTORIAL12/FACTORIAL8,
		[1] = FACTORIAL12/FACTORIAL8,
		[2] = FACTORIAL12/FACTORIAL8,
		[3] = POW2TO11,
		[4] = FACTORIAL8,
		[5] = POW3TO7,
		[6] = FACTORIAL6,
		[7] = NMOVES
	};

	strcpy(fname, tabledir);
	strcat(fname, "/ttables");

	if ((f = fopen(fname, "wb")) == NULL)
		return false;

	for (m = 0; m < NTRANS; m++) {
		r = r && fwrite(epose_ttable[m], b, me[0], f) == me[0];
		r = r && fwrite(eposs_ttable[m], b, me[1], f) == me[1];
		r = r && fwrite(eposm_ttable[m], b, me[2], f) == me[2];
		r = r && fwrite(eo_ttable[m],    b, me[3], f) == me[3];
		r = r && fwrite(cp_ttable[m],    b, me[4], f) == me[4];
		r = r && fwrite(co_ttable[m],    b, me[5], f) == me[5];
		r = r && fwrite(cpos_ttable[m],  b, me[6], f) == me[6];
		r = r && fwrite(moves_ttable[m], b, me[7], f) == me[7];
	}

	fclose(f);
	return r;
}

/* Public functions **********************************************************/

Cube
apply_trans(Trans t, Cube cube)
{
	/*init_trans();*/

	int aux_epos[3] = { cube.epose, cube.eposs, cube.eposm };
	int aux_eo[3]   = { cube.eoud,  cube.eorl,  cube.eofb  };
	int aux_co[3]   = { cube.coud,  cube.corl,  cube.cofb  };

	return (Cube) {
		.epose = epose_ttable[t][aux_epos[epose_source[t]]],
		.eposs = eposs_ttable[t][aux_epos[eposs_source[t]]],
		.eposm = eposm_ttable[t][aux_epos[eposm_source[t]]],
		.eofb  = eo_ttable[t][aux_eo[eofb_source[t]]],
		.eorl  = eo_ttable[t][aux_eo[eorl_source[t]]],
		.eoud  = eo_ttable[t][aux_eo[eoud_source[t]]],
		.coud  = co_ttable[t][aux_co[coud_source[t]]],
		.corl  = co_ttable[t][aux_co[corl_source[t]]],
		.cofb  = co_ttable[t][aux_co[cofb_source[t]]],
		.cp    = cp_ttable[t][cube.cp],
		.cpos  = cpos_ttable[t][cube.cpos]
	};
}

Trans
inverse_trans(Trans t)
{
	static Trans inverse_trans_aux[NTRANS] = {
		[uf] = uf, [ur] = ul, [ul] = ur, [ub] = ub,
		[df] = df, [dr] = dr, [dl] = dl, [db] = db,
		[rf] = lf, [rd] = bl, [rb] = rb, [ru] = fr,
		[lf] = rf, [ld] = br, [lb] = lb, [lu] = fl,
		[fu] = fu, [fr] = ru, [fd] = bu, [fl] = lu,
		[bu] = fd, [br] = ld, [bd] = bd, [bl] = rd,

		[uf_mirror] = uf_mirror, [ur_mirror] = ur_mirror,
		[ul_mirror] = ul_mirror, [ub_mirror] = ub_mirror,
		[df_mirror] = df_mirror, [dr_mirror] = dl_mirror,
		[dl_mirror] = dr_mirror, [db_mirror] = db_mirror,
		[rf_mirror] = rf_mirror, [rd_mirror] = br_mirror,
		[rb_mirror] = lb_mirror, [ru_mirror] = fl_mirror,
		[lf_mirror] = lf_mirror, [ld_mirror] = bl_mirror,
		[lb_mirror] = rb_mirror, [lu_mirror] = fr_mirror,
		[fu_mirror] = fu_mirror, [fr_mirror] = lu_mirror,
		[fd_mirror] = bu_mirror, [fl_mirror] = ru_mirror,
		[bu_mirror] = fd_mirror, [br_mirror] = rd_mirror,
		[bd_mirror] = bd_mirror, [bl_mirror] = ld_mirror
	};

	return inverse_trans_aux[t];
}

Alg *
rotation_alg(Trans t)
{
	int i;

	static Alg *rotation_alg_arr[NROTATIONS];
	static bool initialized = false;

	if (!initialized) {
		for (i = 0; i < NROTATIONS; i++)
			rotation_alg_arr[i] = new_alg(rotation_alg_string[i]);

		initialized = true;
	}

	return rotation_alg_arr[t % NROTATIONS];
}

void
transform_alg(Trans t, Alg *alg)
{
	int i;

	/*init_trans();*/
	
	for (i = 0; i < alg->len; i++)
		alg->move[i] = moves_ttable[t][alg->move[i]];
}

void
init_trans(void)
{
	static bool initialized = false;
	if (initialized)
		return;
	initialized = true;

	init_moves();

	Cube aux, cube, c[3];
	CubeArray epcp;
	int i, eparr[12], eoarr[12], cparr[8], coarr[8];
	unsigned int ui;
	Move mi, move;
	Trans m;

	/* Compute sources */
	for (i = 0; i < NTRANS; i++) {
		cube = apply_alg(rotation_alg(i), (Cube){0});

		epose_source[i] = edge_slice(what_edge_at(cube, FR));
		eposs_source[i] = edge_slice(what_edge_at(cube, UR));
		eposm_source[i] = edge_slice(what_edge_at(cube, UF));
		eofb_source[i]  = what_center_at(cube, F_center)/2;
		eorl_source[i]  = what_center_at(cube, R_center)/2;
		eoud_source[i]  = what_center_at(cube, U_center)/2;
		coud_source[i]  = what_center_at(cube, U_center)/2;
		cofb_source[i]  = what_center_at(cube, F_center)/2;
		corl_source[i]  = what_center_at(cube, R_center)/2;
	}

	if (read_ttables_file())
		return;

	fprintf(stderr, "Cannot load %s, generating it\n", "ttables"); 

	/* Initialize tables */
	for (m = 0; m < NTRANS; m++) {
		epcp = (CubeArray){ .ep = eparr, .cp = cparr };
		cube = apply_alg(rotation_alg(m), (Cube){0});
		cube_to_arrays(cube, &epcp, pf_epcp);
		if (m >= NROTATIONS) {
			apply_permutation(ep_mirror, eparr, 12);
			apply_permutation(cp_mirror, cparr,  8);
		}

		for (ui = 0; ui < FACTORIAL12/FACTORIAL8; ui++) {
			c[0] = admissible_ep((Cube){ .epose = ui }, pf_e);
			c[1] = admissible_ep((Cube){ .eposs = ui }, pf_s);
			c[2] = admissible_ep((Cube){ .eposm = ui }, pf_m);

			cube = rotate_via_compose(m,c[epose_source[m]],pf_ep);
			epose_ttable[m][ui] = cube.epose;

			cube = rotate_via_compose(m,c[eposs_source[m]],pf_ep);
			eposs_ttable[m][ui] = cube.eposs;

			cube = rotate_via_compose(m,c[eposm_source[m]],pf_ep);
			eposm_ttable[m][ui] = cube.eposm;
		}
		for (ui = 0; ui < POW2TO11; ui++ ) {
			int_to_sum_zero_array(ui, 2, 12, eoarr);
			apply_permutation(eparr, eoarr, 12);
			eo_ttable[m][ui] = digit_array_to_int(eoarr, 11, 2);
		}
		for (ui = 0; ui < POW3TO7; ui++) {
			int_to_sum_zero_array(ui, 3, 8, coarr);
			apply_permutation(cparr, coarr, 8);
			co_ttable[m][ui] = digit_array_to_int(coarr, 7, 3);
			if (m >= NROTATIONS)
				co_ttable[m][ui] =
					invert_digits(co_ttable[m][ui], 3, 7);
		}
		for (ui = 0; ui < FACTORIAL8; ui++) {
			cube = (Cube){ .cp = ui };
			cube = rotate_via_compose(m, cube, pf_cp);
			cp_ttable[m][ui] = cube.cp;
		}
		for (ui = 0; ui < FACTORIAL6; ui++) {
			cube = (Cube){ .cpos = ui };
			cube = rotate_via_compose(m, cube, pf_cpos);
			cpos_ttable[m][ui] = cube.cpos;
		}
		for (mi = 0; mi < NMOVES; mi++) {
			/* Old version:
			 *
			aux = apply_trans(m, apply_move(mi, (Cube){0}));
			for (move = 0; move < NMOVES; move++) {
				cube = apply_move(inverse_move(move), aux);
				mirr = apply_trans(uf_mirror, cube);
				if (is_solved(cube) || is_solved(mirr))
					moves_ttable[m][mi] = move;
			}
			*/

			aux = apply_trans(m, apply_move(mi, (Cube){0}));
			for (move = 0; move < NMOVES; move++) {
				cube = apply_move(inverse_move(move), aux);
				if (is_solved(cube)) {
					moves_ttable[m][mi] = move;
					break;
				}
			}
		}
	}

	if (!write_ttables_file())
		fprintf(stderr, "Error writing ttables\n");
}

