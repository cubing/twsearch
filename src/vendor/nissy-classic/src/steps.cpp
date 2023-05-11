#include "steps.h"

#define UPDATECHECKSTOP(a, b, c)     if ((a=(MAX((a),(b))))>(c)) return (a);

/* Checkers, estimators and validators ***************************************/

static bool             check_centers(Cube cube);
static bool             check_coud_HTM(Cube cube);
static bool             check_coud_URF(Cube cube);
static bool             check_corners_HTM(Cube cube);
static bool             check_corners_URF(Cube cube);
static bool             check_cornershtr(Cube cube);
static bool             check_eofb(Cube cube);
static bool             check_drud(Cube cube);
static bool             check_htr(Cube cube);

static int              estimate_eofb_HTM(DfsArg *arg);
static int              estimate_coud_HTM(DfsArg *arg);
static int              estimate_coud_URF(DfsArg *arg);
static int              estimate_corners_HTM(DfsArg *arg);
static int              estimate_cornershtr_HTM(DfsArg *arg);
static int              estimate_corners_URF(DfsArg *arg);
static int              estimate_cornershtr_URF(DfsArg *arg);
static int              estimate_drud_HTM(DfsArg *arg);
static int              estimate_drud_eofb(DfsArg *arg);
static int              estimate_dr_eofb(DfsArg *arg);
static int              estimate_drudfin_drud(DfsArg *arg);
static int              estimate_htr_drud(DfsArg *arg);
static int              estimate_cp_drud(DfsArg *arg);
static int              estimate_htrfin_htr(DfsArg *arg);
static int              estimate_nxopt31_HTM(DfsArg *arg);
static int              estimate_light_HTM(DfsArg *arg);

static int              estimate_nxoptlike(DfsArg *arg, PruneData *pd);

static bool             always_valid(Alg *alg);
static bool             validate_singlecw_ending(Alg *alg);

/* Pre-transformation detectors **********************************************/

static int              detect_pretrans_eofb(Cube cube, Trans *ret);
static int              detect_pretrans_drud(Cube cube, Trans *ret);
static int              detect_pretrans_void_3axis(Cube cube, Trans *ret);

/* Messages for when cube is not ready ***************************************/

static char check_centers_msg[100] = "cube must be oriented (centers solved)";
static char check_eo_msg[100]      = "EO must be solved on given axis";
static char check_dr_msg[100]      = "DR must be solved on given axis";
static char check_htr_msg[100]     = "HTR must be solved";
static char check_drany_msg[100]   = "DR must be solved on at least one axis";
static char check_co_msg[100]      = "CO must be solved on the given axis";
static char check_coany_msg[100]   = "CO must be solved on at least one axis";

/* Steps *********************************************************************/

/* Optimal solvers *******************/

Step
optimal_HTM = {
	.shortname = "optimal",
	.name      = "Optimal solve (in HTM)",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_nxopt31_HTM,
	.ready     = check_centers,
	.ready_msg = check_centers_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_HTM,

	.pre_trans = uf,

	.tables    = {&pd_nxopt31_HTM, &pd_corners_HTM},
	.ntables   = 2,
};

Step
optimal_light_HTM = {
	.shortname = "light",
	.name      = "Optimal solve (in HTM), small table (500Mb RAM total)",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_light_HTM,
	.ready     = check_centers,
	.ready_msg = check_centers_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_HTM,

	.pre_trans = uf,

	.tables    = {&pd_drud_sym16_HTM, &pd_corners_HTM},
	.ntables   = 2,
};

/* Optimal after EO ******************/

Step
eofin_eo = {
	.shortname = "eofin",
	.name      = "Optimal solve after EO without breaking EO (detected)",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_nxopt31_HTM,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_eofb,

	.detect    = detect_pretrans_eofb,

	.tables    = {&pd_nxopt31_HTM, &pd_corners_HTM},
	.ntables   = 2,
};

Step
eofbfin_eofb = {
	.shortname = "eofbfin",
	.name      = "Optimal after EO on F/B without breaking EO",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_nxopt31_HTM,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_eofb,

	.pre_trans = uf,

	.tables    = {&pd_nxopt31_HTM, &pd_corners_HTM},
	.ntables   = 2,
};

Step
eorlfin_eorl = {
	.shortname = "eorlfin",
	.name      = "Optimal after EO on R/L without breaking EO",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_nxopt31_HTM,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_eofb,

	.pre_trans = ur,

	.tables    = {&pd_nxopt31_HTM, &pd_corners_HTM},
	.ntables   = 2,
};

Step
eoudfin_eoud = {
	.shortname = "eoudfin",
	.name      = "Optimal after EO on U/D without breaking EO",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_nxopt31_HTM,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_eofb,

	.pre_trans = fd,

	.tables    = {&pd_nxopt31_HTM, &pd_corners_HTM},
	.ntables   = 2,
};

/* EO steps **************************/
Step
eoany_HTM = {
	.shortname = "eo",
	.name      = "EO on any axis",

	.final     = false,
	.is_done   = check_eofb,
	.estimate  = estimate_eofb_HTM,
	.ready     = check_centers,
	.ready_msg = check_centers_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.detect    = detect_pretrans_void_3axis,

	.tables    = {&pd_eofb_HTM},
	.ntables   = 1,
};

Step
eofb_HTM = {
	.shortname = "eofb",
	.name      = "EO on F/B",

	.final     = false,
	.is_done   = check_eofb,
	.estimate  = estimate_eofb_HTM,
	.ready     = check_centers,
	.ready_msg = check_centers_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.pre_trans = uf,

	.tables    = {&pd_eofb_HTM},
	.ntables   = 1,
};

Step
eorl_HTM = {
	.shortname = "eorl",
	.name      = "EO on R/L",

	.final     = false,
	.is_done   = check_eofb,
	.estimate  = estimate_eofb_HTM,
	.ready     = check_centers,
	.ready_msg = check_centers_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.pre_trans = ur,

	.tables    = {&pd_eofb_HTM},
	.ntables   = 1,
};

Step
eoud_HTM = {
	.shortname = "eoud",
	.name      = "EO on U/D",

	.final     = false,
	.is_done   = check_eofb,
	.estimate  = estimate_eofb_HTM,
	.ready     = check_centers,
	.ready_msg = check_centers_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.pre_trans = fd,

	.tables    = {&pd_eofb_HTM},
	.ntables   = 1,
};

/* CO steps **************************/
Step
coany_HTM = {
	.shortname = "co",
	.name      = "CO on any axis",

	.final     = false,
	.is_done   = check_coud_HTM,
	.estimate  = estimate_coud_HTM,
	.ready     = NULL,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.detect    = detect_pretrans_void_3axis,

	.tables    = {&pd_coud_HTM},
	.ntables   = 1,
};

Step
coud_HTM = {
	.shortname = "coud",
	.name      = "CO on U/D",

	.final     = false,
	.is_done   = check_coud_HTM,
	.estimate  = estimate_coud_HTM,
	.ready     = NULL,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.pre_trans = uf,

	.tables    = {&pd_coud_HTM},
	.ntables   = 1,
};

Step
corl_HTM = {
	.shortname = "corl",
	.name      = "CO on R/L",

	.final     = false,
	.is_done   = check_coud_HTM,
	.estimate  = estimate_coud_HTM,
	.ready     = NULL,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.pre_trans = rf,

	.tables    = {&pd_coud_HTM},
	.ntables   = 1,
};

Step
cofb_HTM = {
	.shortname = "cofb",
	.name      = "CO on F/B",

	.final     = false,
	.is_done   = check_coud_HTM,
	.estimate  = estimate_coud_HTM,
	.ready     = NULL,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.pre_trans = fd,

	.tables    = {&pd_coud_HTM},
	.ntables   = 1,
};

Step
coany_URF = {
	.shortname = "co-URF",
	.name      = "CO any axis (URF moveset)",

	.final     = false,
	.is_done   = check_coud_URF,
	.estimate  = estimate_coud_URF,
	.ready     = NULL,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_URF,

	.detect    = detect_pretrans_void_3axis,

	.tables    = {&pd_coud_HTM},
	.ntables   = 1,
};

Step
coud_URF = {
	.shortname = "coud-URF",
	.name      = "CO on U/D (URF moveset)",

	.final     = false,
	.is_done   = check_coud_URF,
	.estimate  = estimate_coud_URF,
	.ready     = NULL,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_URF,

	.pre_trans = uf,

	.tables    = {&pd_coud_HTM},
	.ntables   = 1,
};

Step
corl_URF = {
	.shortname = "corl-URF",
	.name      = "CO on R/L (URF moveset)",

	.final     = false,
	.is_done   = check_coud_URF,
	.estimate  = estimate_coud_URF,
	.ready     = NULL,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_URF,

	.pre_trans = rf,

	.tables    = {&pd_coud_HTM},
	.ntables   = 1,
};

Step
cofb_URF = {
	.shortname = "cofb-URF",
	.name      = "CO on F/B (URF moveset)",

	.final     = false,
	.is_done   = check_coud_URF,
	.estimate  = estimate_coud_URF,
	.ready     = NULL,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_URF,

	.pre_trans = fd,

	.tables    = {&pd_coud_HTM},
	.ntables   = 1,
};

/* Misc corner steps *****************/
Step
cornershtr_HTM = {
	.shortname = "chtr",
	.name      = "Solve corners to HTR state",

	.final     = false,
	.is_done   = check_cornershtr,
	.estimate  = estimate_cornershtr_HTM,
	.ready     = NULL,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.pre_trans = uf,

	.tables    = {&pd_cornershtr_HTM},
	.ntables   = 1,
};

Step
cornershtr_URF = {
	.shortname = "chtr-URF",
	.name      = "Solve corners to HTR state (URF moveset)",

	.final     = false,
	.is_done   = check_cornershtr,
	.estimate  = estimate_cornershtr_URF,
	.ready     = NULL,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_URF,

	.pre_trans = uf,

	.tables    = {&pd_cornershtr_HTM},
	.ntables   = 1,
};

Step
corners_HTM = {
	.shortname = "corners",
	.name      = "Solve corners",

	.final     = true,
	.is_done   = check_corners_HTM,
	.estimate  = estimate_corners_HTM,
	.ready     = NULL,
	.is_valid  = always_valid,
	.moveset   = &moveset_HTM,

	.pre_trans = uf,

	.tables    = {&pd_corners_HTM},
	.ntables   = 1,
};

Step
corners_URF = {
	.shortname = "corners-URF",
	.name      = "Solve corners (URF moveset)",

	.final     = true, /* TODO: check if this works with reorient */
	.is_done   = check_corners_URF,
	.estimate  = estimate_corners_URF,
	.ready     = NULL,
	.is_valid  = always_valid,
	.moveset   = &moveset_URF,

	.pre_trans = uf,

	.tables    = {&pd_corners_HTM},
	.ntables   = 1,
};

/* DR steps **************************/
Step
drany_HTM = {
	.shortname = "dr",
	.name      = "DR on any axis",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_drud_HTM,
	.ready     = check_centers,
	.ready_msg = check_centers_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.detect    = detect_pretrans_void_3axis,

	.tables    = {&pd_drud_sym16_HTM},
	.ntables   = 1,
};

Step
drud_HTM = {
	.shortname = "drud",
	.name      = "DR on U/D",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_drud_HTM,
	.ready     = check_centers,
	.ready_msg = check_centers_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.pre_trans = uf,

	.tables    = {&pd_drud_sym16_HTM},
	.ntables   = 1,
};

Step
drrl_HTM = {
	.shortname = "drrl",
	.name      = "DR on R/L",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_drud_HTM,
	.ready     = check_centers,
	.ready_msg = check_centers_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.pre_trans = rf,

	.tables    = {&pd_drud_sym16_HTM},
	.ntables   = 1,
};

Step
drfb_HTM = {
	.shortname = "drfb",
	.name      = "DR on F/B",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_drud_HTM,
	.ready     = check_centers,
	.ready_msg = check_centers_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_HTM,

	.pre_trans = fd,

	.tables    = {&pd_drud_sym16_HTM},
	.ntables   = 1,
};

/* DR from EO */
Step
dr_eo = {
	.shortname = "dr-eo",
	.name      = "DR without breaking EO (automatically detected)",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_dr_eofb,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_eofb,

	.detect    = detect_pretrans_eofb,

	.tables    = {&pd_drud_eofb},
	.ntables   = 1,
};

Step
dr_eofb = {
	.shortname = "dr-eofb",
	.name      = "DR on U/D or R/L without breaking EO on F/B",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_dr_eofb,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_eofb,

	.pre_trans = uf,

	.tables    = {&pd_drud_eofb},
	.ntables   = 1,
};

Step
dr_eorl = {
	.shortname = "dr-eorl",
	.name      = "DR on U/D or F/B without breaking EO on R/L",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_dr_eofb,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_eofb,

	.pre_trans = ur,

	.tables    = {&pd_drud_eofb},
	.ntables   = 1,
};

Step
dr_eoud = {
	.shortname = "dr-eoud",
	.name      = "DR on R/L or F/B without breaking EO on U/D",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_dr_eofb,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_eofb,

	.pre_trans = fd,

	.tables    = {&pd_drud_eofb},
	.ntables   = 1,
};

Step
drud_eofb = {
	.shortname = "drud-eofb",
	.name      = "DR on U/D without breaking EO on F/B",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_drud_eofb,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_eofb,

	.pre_trans = uf,

	.tables    = {&pd_drud_eofb},
	.ntables   = 1,
};

Step
drrl_eofb = {
	.shortname = "drrl-eofb",
	.name      = "DR on R/L without breaking EO on F/B",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_drud_eofb,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_eofb,

	.pre_trans = rf,

	.tables    = {&pd_drud_eofb},
	.ntables   = 1,
};

Step
drud_eorl = {
	.shortname = "drud-eorl",
	.name      = "DR on U/D without breaking EO on R/L",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_drud_eofb,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_eofb,

	.pre_trans = ur,

	.tables    = {&pd_drud_eofb},
	.ntables   = 1,
};

Step
drfb_eorl = {
	.shortname = "drfb-eorl",
	.name      = "DR on F/B without breaking EO on R/L",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_drud_eofb,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_eofb,

	.pre_trans = fr,

	.tables    = {&pd_drud_eofb},
	.ntables   = 1,
};

Step
drfb_eoud = {
	.shortname = "drfb-eoud",
	.name      = "DR on F/B without breaking EO on U/D",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_drud_eofb,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_eofb,

	.pre_trans = fd,

	.tables    = {&pd_drud_eofb},
	.ntables   = 1,
};

Step
drrl_eoud = {
	.shortname = "drrl-eoud",
	.name      = "DR on R/L without breaking EO on U/D",

	.final     = false,
	.is_done   = check_drud,
	.estimate  = estimate_drud_eofb,
	.ready     = check_eofb,
	.ready_msg = check_eo_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_eofb,

	.pre_trans = rd,

	.tables    = {&pd_drud_eofb},
	.ntables   = 1,
};

/* DR finish steps */
Step
dranyfin_DR = {
	.shortname = "drfin",
	.name      = "DR finish on any axis without breaking DR",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_drudfin_drud,
	.ready     = check_drud,
	.ready_msg = check_drany_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_drud,

	.detect    = detect_pretrans_drud,

	.tables    = {&pd_drudfin_noE_sym16_drud},
	.ntables   = 1,
};

Step
drudfin_drud = {
	.shortname = "drudfin",
	.name      = "DR finish on U/D without breaking DR",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_drudfin_drud,
	.ready     = check_drud,
	.ready_msg = check_dr_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_drud,

	.pre_trans = uf,

	.tables    = {&pd_drudfin_noE_sym16_drud},
	.ntables   = 1,
};

Step
drrlfin_drrl = {
	.shortname = "drrlfin",
	.name      = "DR finish on R/L without breaking DR",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_drudfin_drud,
	.ready     = check_drud,
	.ready_msg = check_dr_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_drud,

	.pre_trans = rf,

	.tables    = {&pd_drudfin_noE_sym16_drud},
	.ntables   = 1,
};

Step
drfbfin_drfb = {
	.shortname = "drfbfin",
	.name      = "DR finish on F/B without breaking DR",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_drudfin_drud,
	.ready     = check_drud,
	.ready_msg = check_dr_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_drud,

	.pre_trans = fd,

	.tables    = {&pd_drudfin_noE_sym16_drud},
	.ntables   = 1,
};

/* HTR from DR */
Step
htr_any = {
	.shortname = "htr",
	.name      = "HTR from DR",

	.final     = false,
	.is_done   = check_htr,
	.estimate  = estimate_htr_drud,
	.ready     = check_drud,
	.ready_msg = check_drany_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_drud,

	.detect    = detect_pretrans_drud,

	.tables    = {&pd_htr_drud},
	.ntables   = 1,
};

Step
htr_drud = {
	.shortname = "htr-drud",
	.name      = "HTR from DR on U/D",

	.final     = false,
	.is_done   = check_htr,
	.estimate  = estimate_htr_drud,
	.ready     = check_drud,
	.ready_msg = check_dr_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_drud,

	.pre_trans = uf,

	.tables    = {&pd_htr_drud},
	.ntables   = 1,
};

Step
htr_drrl = {
	.shortname = "htr-drrl",
	.name      = "HTR from DR on R/L",

	.final     = false,
	.is_done   = check_htr,
	.estimate  = estimate_htr_drud,
	.ready     = check_drud,
	.ready_msg = check_dr_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_drud,

	.pre_trans = rf,

	.tables    = {&pd_htr_drud},
	.ntables   = 1,
};

Step
htr_drfb = {
	.shortname = "htr-drfb",
	.name      = "HTR from DR on F/B",

	.final     = false,
	.is_done   = check_htr,
	.estimate  = estimate_htr_drud,
	.ready     = check_drud,
	.ready_msg = check_dr_msg,
	.is_valid  = validate_singlecw_ending,
	.moveset   = &moveset_drud,

	.pre_trans = fd,

	.tables    = {&pd_htr_drud},
	.ntables   = 1,
};

/* Corners from DR */
Step
corners_dr_any = {
	.shortname = "corners-dr",
	.name      = "Solve corners from DR",

	.final     = false,
	.is_done   = check_corners_HTM,
	.estimate  = estimate_cp_drud,
	.ready     = check_coud_HTM,
	.ready_msg = check_coany_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_drud,

	.detect    = detect_pretrans_drud,

	.tables    = {&pd_cp_drud},
	.ntables   = 1,
};

Step
corners_drud = {
	.shortname = "corners-drud",
	.name      = "Solve corners from DR on U/D",

	.final     = false,
	.is_done   = check_corners_HTM,
	.estimate  = estimate_cp_drud,
	.ready     = check_coud_HTM,
	.ready_msg = check_co_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_drud,

	.pre_trans = uf,

	.tables    = {&pd_cp_drud},
	.ntables   = 1,
};

Step
corners_drrl = {
	.shortname = "corners-drrl",
	.name      = "Solve corners from DR on R/L",

	.final     = false,
	.is_done   = check_corners_HTM,
	.estimate  = estimate_cp_drud,
	.ready     = check_coud_HTM,
	.ready_msg = check_co_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_drud,

	.pre_trans = rf,

	.tables    = {&pd_cp_drud},
	.ntables   = 1,
};

Step
corners_drfb = {
	.shortname = "corners-drfb",
	.name      = "Solve corners from DR on F/B",

	.final     = false,
	.is_done   = check_corners_HTM,
	.estimate  = estimate_cp_drud,
	.ready     = check_coud_HTM,
	.ready_msg = check_co_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_drud,

	.pre_trans = fd,

	.tables    = {&pd_cp_drud},
	.ntables   = 1,
};

/* HTR finish */
Step
htrfin_htr = {
	.shortname = "htrfin",
	.name      = "HTR finish without breaking HTR",

	.final     = true,
	.is_done   = is_solved,
	.estimate  = estimate_htrfin_htr,
	.ready     = check_htr,
	.ready_msg = check_htr_msg,
	.is_valid  = always_valid,
	.moveset   = &moveset_htr,

	.pre_trans = uf,

	.tables    = {&pd_htrfin_htr},
	.ntables   = 1,
};

Step *steps[] = {
	&optimal_HTM, /* first is default */
	&optimal_light_HTM,

	&eofin_eo,
	&eofbfin_eofb,
	&eorlfin_eorl,
	&eoudfin_eoud,

	&eoany_HTM,
	&eofb_HTM,
	&eorl_HTM,
	&eoud_HTM,

	&coany_HTM,
	&coud_HTM,
	&corl_HTM,
	&cofb_HTM,

	&coany_URF,
	&coud_URF,
	&corl_URF,
	&cofb_URF,

	&drany_HTM,
	&drud_HTM,
	&drrl_HTM,
	&drfb_HTM,

	&dr_eo,
	&dr_eofb,
	&dr_eorl,
	&dr_eoud,
	&drud_eofb,
	&drrl_eofb,
	&drud_eorl,
	&drfb_eorl,
	&drfb_eoud,
	&drrl_eoud,

	&dranyfin_DR,
	&drudfin_drud,
	&drrlfin_drrl,
	&drfbfin_drfb,

	&htr_any,
	&htr_drud,
	&htr_drrl,
	&htr_drfb,

	&corners_dr_any,
	&corners_drud,
	&corners_drrl,
	&corners_drfb,

	&htrfin_htr,

	&cornershtr_HTM,
	&cornershtr_URF,
	&corners_HTM,
	&corners_URF,

	NULL
};

/* Checkers, estimators and validators ***************************************/

static bool
check_centers(Cube cube)
{
	return cube.cpos == 0;
}

static bool
check_coud_HTM(Cube cube)
{
	return cube.coud == 0;
}

static bool
check_coud_URF(Cube cube)
{
	Cube c2, c3;

	c2 = apply_move(z, cube);
	c3 = apply_move(x, cube);

	return cube.coud == 0 || c2.coud == 0 || c3.coud == 0;
}

static bool
check_corners_URF(Cube cube)
{
	Cube c;
	Trans i;

	for (i = 0; i < NROTATIONS; i++) {
		c = apply_alg(rotation_alg(i), cube);
		if (c.cp && c.coud)
			return true;
	}

	return false;
}

static bool
check_corners_HTM(Cube cube)
{
	return cube.cp == 0 && cube.coud == 0;
}

static bool
check_cornershtr(Cube cube)
{
	return coord_cornershtr.index(cube) == 0;
}

static bool
check_eofb(Cube cube)
{
	return cube.eofb == 0;
}

static bool
check_drud(Cube cube)
{
	return cube.eofb == 0 && cube.eorl == 0 && cube.coud == 0;
}

static bool
check_htr(Cube cube)
{
	return check_drud(cube) && coord_htr_drud.index(cube) == 0;
}

static int
estimate_eofb_HTM(DfsArg *arg)
{
	return ptableval(&pd_eofb_HTM, arg->cube);
}

static int
estimate_coud_HTM(DfsArg *arg)
{
	return ptableval(&pd_coud_HTM, arg->cube);
}

static int
estimate_coud_URF(DfsArg *arg)
{
	/* TODO: I can improve this by checking first the orientation of
	 * the corner in DBL and use that as a reference */

	Cube c;

	c = arg->cube;

	int ud = estimate_coud_HTM(arg);
	arg->cube = apply_move(z, c);
	int rl = estimate_coud_HTM(arg);
	arg->cube = apply_move(x, c);
	int fb = estimate_coud_HTM(arg);

	arg->cube = c;

	return MIN(ud, MIN(rl, fb));
}

static int
estimate_corners_HTM(DfsArg *arg)
{
	return ptableval(&pd_corners_HTM, arg->cube);
}

static int
estimate_cornershtr_HTM(DfsArg *arg)
{
	return ptableval(&pd_cornershtr_HTM, arg->cube);
}

static int
estimate_cornershtr_URF(DfsArg *arg)
{
	/* TODO: I can improve this by checking first the corner in DBL
	 * and use that as a reference */

	int ret;
	Cube c;
	Trans i;

	c = arg->cube;
	ret = 15;

	for (i = 0; i < NROTATIONS; i++) {
		arg->cube = apply_alg(rotation_alg(i), c);
		ret = MIN(ret, estimate_cornershtr_HTM(arg));
	}

	arg->cube = c;

	return ret;
}

static int
estimate_corners_URF(DfsArg *arg)
{
	/* TODO: I can improve this by checking first the corner in DBL
	 * and use that as a reference */

	int ret;
	Cube c;
	Trans i;

	c = arg->cube;
	ret = 15;

	for (i = 0; i < NROTATIONS; i++) {
		arg->cube = apply_alg(rotation_alg(i), c);
		ret = MIN(ret, estimate_corners_HTM(arg));
	}

	arg->cube = c;

	return ret;
}

static int
estimate_drud_HTM(DfsArg *arg)
{
	return ptableval(&pd_drud_sym16_HTM, arg->cube);
}

static int
estimate_drud_eofb(DfsArg *arg)
{
	return ptableval(&pd_drud_eofb, arg->cube);
}

static int
estimate_dr_eofb(DfsArg *arg)
{
	int r1, r2;

	r1 = ptableval(&pd_drud_eofb, arg->cube);
	r2 = ptableval(&pd_drud_eofb, apply_trans(rf, arg->cube));

	return MIN(r1, r2);
}

static int
estimate_drudfin_drud(DfsArg *arg)
{
	int val = ptableval(&pd_drudfin_noE_sym16_drud, arg->cube);

	if (val != 0)
		return val;

	return arg->cube.epose % 24 == 0 ? 0 : 1;
}

static int
estimate_htr_drud(DfsArg *arg)
{
	return ptableval(&pd_htr_drud, arg->cube);
}

static int
estimate_cp_drud(DfsArg *arg)
{
	return ptableval(&pd_cp_drud, arg->cube);
}

static int
estimate_htrfin_htr(DfsArg *arg)
{
	return ptableval(&pd_htrfin_htr, arg->cube);
}

static int
estimate_nxopt31_HTM(DfsArg *arg)
{
	return estimate_nxoptlike(arg, &pd_nxopt31_HTM);
}

/* TODO: also use generic procedure for this */
static int
estimate_light_HTM(DfsArg *arg)
{
	int target, ret;
	Cube aux;

	static const uint64_t udmask = (1<<U) | (1<<U2) | (1<<U3) |
				       (1<<D) | (1<<D2) | (1<<D3);
	static const uint64_t rlmask = (1<<R) | (1<<R2) | (1<<R3) |
				       (1<<L) | (1<<L2) | (1<<L3);
	static const uint64_t fbmask = (1<<F) | (1<<F2) | (1<<F3) |
				       (1<<B) | (1<<B2) | (1<<B3);
	static const uint64_t htmask = (1<<U2) | (1<<D2) |
				       (1<<R2) | (1<<L2) |
				       (1<<F2) | (1<<B2);

	ret              = -1;
	target           = arg->d - arg->current_alg->len;
	arg->inverse     = (Cube){0};
	arg->badmovesinv = 0;
	arg->badmoves    = 0;

	/* Corners */
	arg->ed->corners = ptableval(&pd_corners_HTM, arg->cube);
	UPDATECHECKSTOP(ret, arg->ed->corners, target);

	/* Normal probing */
	arg->ed->normal_ud = ptableval(&pd_drud_sym16_HTM, arg->cube);
	UPDATECHECKSTOP(ret, arg->ed->normal_ud, target);
	aux = apply_trans(fd, arg->cube);
	arg->ed->normal_fb = ptableval(&pd_drud_sym16_HTM, aux);
	UPDATECHECKSTOP(ret, arg->ed->normal_fb, target);
	aux = apply_trans(rf, arg->cube);
	arg->ed->normal_rl = ptableval(&pd_drud_sym16_HTM, aux);
	UPDATECHECKSTOP(ret, arg->ed->normal_rl, target);

	/* If ret == 0, it's solved (corners + triple slice solved) */
	if (ret == 0)
		return is_solved(arg->cube) ? 0 : 1;

	/* Michel de Bondt's trick*/
	if (arg->ed->normal_ud == arg->ed->normal_fb &&
	    arg->ed->normal_fb == arg->ed->normal_rl) {
		UPDATECHECKSTOP(ret, arg->ed->normal_ud + 1, target);
	}

	/* Inverse probing */
	if (!((1<<arg->last1) & htmask)) {
		aux = arg->inverse = inverse_cube(arg->cube);
		if (!((1<<arg->last1) & udmask) || (arg->ed->inverse_ud==-1)) {
			arg->ed->inverse_ud =
			    ptableval(&pd_drud_sym16_HTM, aux);
		}
		UPDATECHECKSTOP(ret, arg->ed->inverse_ud, target);
		if (!((1<<arg->last1) & fbmask) || (arg->ed->inverse_fb==-1)) {
			aux = apply_trans(fd, arg->inverse);
			arg->ed->inverse_fb =
			    ptableval(&pd_drud_sym16_HTM, aux);
		}
		UPDATECHECKSTOP(ret, arg->ed->inverse_fb, target);
		if (!((1<<arg->last1) & rlmask) || (arg->ed->inverse_rl==-1)) {
			aux = apply_trans(rf, arg->inverse);
			arg->ed->inverse_rl =
			    ptableval(&pd_drud_sym16_HTM, aux);
		}
		UPDATECHECKSTOP(ret, arg->ed->inverse_rl, target);
	} else {
		UPDATECHECKSTOP(ret, arg->ed->inverse_ud, target);
		UPDATECHECKSTOP(ret, arg->ed->inverse_fb, target);
		UPDATECHECKSTOP(ret, arg->ed->inverse_rl, target);
	}

	/* Michel de Bondt's trick*/
	if (arg->ed->inverse_ud == arg->ed->inverse_fb &&
	    arg->ed->inverse_fb == arg->ed->inverse_rl) {
		UPDATECHECKSTOP(ret, arg->ed->inverse_ud + 1, target);
	}

	/* nxopt trick + half turn trick */
	if (arg->ed->normal_ud == target)
		arg->badmovesinv |= udmask | htmask;
	if (arg->ed->normal_fb == target)
		arg->badmovesinv |= fbmask | htmask;
	if (arg->ed->normal_rl == target)
		arg->badmovesinv |= rlmask | htmask;

	if (arg->ed->inverse_ud == target)
		arg->badmoves |= udmask | htmask;
	if (arg->ed->inverse_fb == target)
		arg->badmoves |= fbmask | htmask;
	if (arg->ed->inverse_rl == target)
		arg->badmoves |= rlmask | htmask;

	return arg->ed->oldret = ret;
}

static int
estimate_nxoptlike(DfsArg *arg, PruneData *pd)
{
	int target, ret;
	Cube aux;

	static const uint64_t udmask = (1<<U) | (1<<U2) | (1<<U3) |
				       (1<<D) | (1<<D2) | (1<<D3);
	static const uint64_t rlmask = (1<<R) | (1<<R2) | (1<<R3) |
				       (1<<L) | (1<<L2) | (1<<L3);
	static const uint64_t fbmask = (1<<F) | (1<<F2) | (1<<F3) |
				       (1<<B) | (1<<B2) | (1<<B3);

	ret              = -1;
	target           = arg->d - arg->current_alg->len;
	arg->inverse     = (Cube){0};
	arg->badmovesinv = 0;
	arg->badmoves    = 0;

	/* Corners */
	arg->ed->corners = ptableval(&pd_corners_HTM, arg->cube);
	UPDATECHECKSTOP(ret, arg->ed->corners, target);

	/* Normal probing */
	arg->ed->normal_ud = ptableval(pd, arg->cube);
	UPDATECHECKSTOP(ret, arg->ed->normal_ud, target);
	aux = apply_trans(fd, arg->cube);
	arg->ed->normal_fb = ptableval(pd, aux);
	UPDATECHECKSTOP(ret, arg->ed->normal_fb, target);
	aux = apply_trans(rf, arg->cube);
	arg->ed->normal_rl = ptableval(pd, aux);
	UPDATECHECKSTOP(ret, arg->ed->normal_rl, target);

	if (ret == 0)
		return arg->step->is_done(arg->cube) ? 0 : 1;

	/* Michel de Bondt's trick*/
	if (arg->ed->normal_ud == arg->ed->normal_fb &&
	    arg->ed->normal_fb == arg->ed->normal_rl) {
		UPDATECHECKSTOP(ret, arg->ed->normal_ud + 1, target);
	}

	/* Inverse probing */
	aux = arg->inverse = inverse_cube(arg->cube);
	if (!((1<<arg->last1) & udmask) || (arg->ed->inverse_ud == -1)) {
		arg->ed->inverse_ud = ptableval(pd, aux);
	}
	UPDATECHECKSTOP(ret, arg->ed->inverse_ud, target);
	if (!((1<<arg->last1) & fbmask) || (arg->ed->inverse_fb == -1)) {
		aux = apply_trans(fd, arg->inverse);
		arg->ed->inverse_fb = ptableval(pd, aux);
	}
	UPDATECHECKSTOP(ret, arg->ed->inverse_fb, target);
	if (!((1<<arg->last1) & rlmask) || (arg->ed->inverse_rl == -1)) {
		aux = apply_trans(rf, arg->inverse);
		arg->ed->inverse_rl = ptableval(pd, aux);
	}
	UPDATECHECKSTOP(ret, arg->ed->inverse_rl, target);

	/* Michel de Bondt's trick*/
	if (arg->ed->inverse_ud == arg->ed->inverse_fb &&
	    arg->ed->inverse_fb == arg->ed->inverse_rl) {
		UPDATECHECKSTOP(ret, arg->ed->inverse_ud + 1, target);
	}

	/* nxopt trick */
	if (arg->ed->normal_ud == target)
		arg->badmovesinv |= udmask;
	if (arg->ed->normal_fb == target)
		arg->badmovesinv |= fbmask;
	if (arg->ed->normal_rl == target)
		arg->badmovesinv |= rlmask;

	if (arg->ed->inverse_ud == target)
		arg->badmoves |= udmask;
	if (arg->ed->inverse_fb == target)
		arg->badmoves |= fbmask;
	if (arg->ed->inverse_rl == target)
		arg->badmoves |= rlmask;

	return arg->ed->oldret = ret;
}


static bool
always_valid(Alg *alg)
{
	return true;
}

static bool
validate_singlecw_ending(Alg *alg)
{
	int i;
	bool nor, inv;
	Move l2 = NULLMOVE, l1 = NULLMOVE, l2i = NULLMOVE, l1i = NULLMOVE;

	for (i = 0; i < alg->len; i++) {
		if (alg->inv[i]) {
			l2i = l1i;
			l1i = alg->move[i];
		} else {
			l2 = l1;
			l1 = alg->move[i];
		}
	}

	nor = l1 ==base_move(l1)  && (!commute(l1, l2) ||l2 ==base_move(l2));
	inv = l1i==base_move(l1i) && (!commute(l1i,l2i)||l2i==base_move(l2i));

	return nor && inv;
}

/* Pre-transformation detectors **********************************************/

static int
detect_pretrans_eofb(Cube cube, Trans *ret)
{
	int i, n;
	static Trans tt[3] = {uf, ur, fd};

	for (i = 0, n = 0; i < 3; i++)
		if (check_eofb(apply_trans(tt[i], cube)))
			ret[n++] = tt[i];

	return n;
}

static int
detect_pretrans_drud(Cube cube, Trans *ret)
{
	int i, n;
	static Trans tt[3] = {uf, fr, rd};

	for (i = 0, n = 0; i < 3; i++)
		if (check_drud(apply_trans(tt[i], cube)))
			ret[n++] = tt[i];

	return n;
}

static int
detect_pretrans_void_3axis(Cube cube, Trans *ret)
{
	ret[0] = uf;
	ret[1] = fr;
	ret[2] = rd;

	return 3;
}

/* Public functions **********************************************************/

void
copy_estimatedata(EstimateData *src, EstimateData *dst)
{
	dst->corners    = src->corners;
	dst->normal_ud  = src->normal_ud;
	dst->normal_fb  = src->normal_fb;
	dst->normal_rl  = src->normal_rl;
	dst->inverse_ud = src->inverse_ud;
	dst->inverse_fb = src->inverse_fb;
	dst->inverse_rl = src->inverse_rl;
	dst->oldret     = src->oldret;
}

void
invert_estimatedata(EstimateData *ed)
{
	swap(&(ed->normal_ud), &(ed->inverse_ud));
	swap(&(ed->normal_fb), &(ed->inverse_fb));
	swap(&(ed->normal_rl), &(ed->inverse_rl));
}

void
reset_estimatedata(EstimateData *ed)
{
	ed->corners    = -1;
	ed->normal_ud  = -1;
	ed->normal_fb  = -1;
	ed->normal_rl  = -1;
	ed->inverse_ud = -1;
	ed->inverse_fb = -1;
	ed->inverse_rl = -1;
	ed->oldret     = -1;
}

void
prepare_step(Step *step, SolveOptions *opts)
{
	int i;

	if (step->final && opts->can_niss) {
		opts->can_niss = false;
		fprintf(stderr, "Step is final, NISS not used (-n ignored)\n");
	}

	for (i = 0; i < step->ntables; i++) {
		genptable(step->tables[i], opts->nthreads);
		if (step->tables[i]->compact)
			genptable(step->tables[i]->fallback, opts->nthreads);
	}
}
