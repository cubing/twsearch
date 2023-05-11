#ifndef CUBETYPES_H
#define CUBETYPES_H

#include <stdbool.h>
#include <inttypes.h>
#include <pthread.h>

#define NMOVES               55 /* Actually 54, but one is NULLMOVE */
#define NTRANS               48
#define NROTATIONS           24
#define entry_group_t        uint8_t /* For pruning tables */

/* Enums *********************************************************************/

typedef enum
center
{
	U_center, D_center,
	R_center, L_center,
	F_center, B_center
} Center;

typedef enum
corner
{
	UFR, UFL, UBL, UBR,
	DFR, DFL, DBL, DBR
} Corner;

typedef enum
edge
{
	UF, UL, UB, UR,
	DF, DL, DB, DR,
	FR, FL, BL, BR
} Edge;

typedef enum
move
{
	NULLMOVE,
	U, U2, U3, D, D2, D3,
	R, R2, R3, L, L2, L3,
	F, F2, F3, B, B2, B3,
	Uw, Uw2, Uw3, Dw, Dw2, Dw3,
	Rw, Rw2, Rw3, Lw, Lw2, Lw3,
	Fw, Fw2, Fw3, Bw, Bw2, Bw3,
	M, M2, M3,
	S, S2, S3,
	E, E2, E3,
	x, x2, x3,
	y, y2, y3,
	z, z2, z3,
} Move;

typedef enum
trans
{
	uf, ur, ub, ul,
	df, dr, db, dl,
	rf, rd, rb, ru,
	lf, ld, lb, lu,
	fu, fr, fd, fl,
	bu, br, bd, bl,
	uf_mirror, ur_mirror, ub_mirror, ul_mirror,
	df_mirror, dr_mirror, db_mirror, dl_mirror,
	rf_mirror, rd_mirror, rb_mirror, ru_mirror,
	lf_mirror, ld_mirror, lb_mirror, lu_mirror,
	fu_mirror, fr_mirror, fd_mirror, fl_mirror,
	bu_mirror, br_mirror, bd_mirror, bl_mirror,
} Trans;


/* Typedefs ******************************************************************/

typedef struct alg                Alg;
typedef struct alglist            AlgList;
typedef struct alglistnode        AlgListNode;
typedef struct block              Block;
typedef struct command            Command;
typedef struct commandargs        CommandArgs;
typedef struct coordinate         Coordinate;
typedef struct cube               Cube;
typedef struct cubearray          CubeArray;
typedef struct dfsarg             DfsArg;
typedef struct estimatedata       EstimateData;
typedef struct moveset            Moveset;
typedef struct piecefilter        PieceFilter;
typedef struct prunedata          PruneData;
typedef struct solveoptions       SolveOptions;
typedef struct step               Step;
typedef struct symdata            SymData;
typedef struct threaddatasolve    ThreadDataSolve;
typedef struct threaddatagenpt    ThreadDataGenpt;

typedef Cube                 (*AntiIndexer)      (uint64_t);
typedef bool                 (*Checker)          (Cube);
typedef uint64_t             (*CoordMover)       (Move, uint64_t);
typedef uint64_t             (*CoordTransformer) (Trans, uint64_t);
typedef int                  (*Estimator)        (DfsArg *);
typedef bool                 (*Validator)        (Alg *);
typedef void                 (*Exec)             (CommandArgs *);
typedef uint64_t             (*Indexer)          (Cube);
typedef CommandArgs *        (*ArgParser)        (int, char **);
typedef int                  (*TransDetector)    (Cube, Trans *);
typedef int                  (*TransFinder)      (uint64_t, Trans *);


/* Structs *******************************************************************/

struct
alg
{
	Move *                    move;
	bool *                    inv;
	int                       len;
	int                       allocated;
};

struct
alglist
{
	AlgListNode *             first;
	AlgListNode *             last;
	int                       len;
};

struct
alglistnode
{
	Alg *                     alg;
	AlgListNode *             next;
};

struct
block
{
	bool                      edge[12];
	bool                      corner[8];
	bool                      center[6];
};

struct
command
{
	char *                    name;
	char *                    usage;
	char *                    description;
	ArgParser                 parse_args;
	Exec                      exec;
};

struct
commandargs
{
	bool                      success;
	Alg *                     scramble;
	SolveOptions *            opts;
	Step *                    step;
	Command *                 command; /* For help */
	int                       n;
	char                      scrtype[20];
	bool                      scrstdin;
	bool                      header;
};

struct
coordinate
{
	Indexer                   index;
	uint64_t                  max;
	CoordMover                move;
	CoordTransformer          transform;
	SymData *                 sd;
	TransFinder               tfind; /* TODO: should be easy to remove */
	Coordinate *              base; /* TODO: part of refactor */
};

struct
cube
{
	int                       epose;
	int                       eposs;
	int                       eposm;
	int                       eofb;
	int                       eorl;
	int                       eoud;
	int                       cp;
	int                       coud;
	int                       cofb;
	int                       corl;
	int                       cpos;
};

struct
cubearray
{
	int *                     ep;
	int *                     eofb;
	int *                     eorl;
	int *                     eoud;
	int *                     cp;
	int *                     coud;
	int *                     corl;
	int *                     cofb;
	int *                     cpos;
};

struct
dfsarg
{
	Step *                    step;
	SolveOptions *            opts;
	Trans                     t;
	Cube                      cube;
	Cube                      inverse;
	int                       d;
	uint64_t                  badmoves;
	uint64_t                  badmovesinv;
	bool                      niss;
	Move                      last1;
	Move                      last2;
	Move                      last1inv;
	Move                      last2inv;
	EstimateData *            ed;
	AlgList *                 sols;
	pthread_mutex_t *         sols_mutex;
	Alg *                     current_alg;
};

struct
estimatedata
{
	int                       corners;
	int                       normal_ud;
	int                       normal_fb;
	int                       normal_rl;
	int                       inverse_ud;
	int                       inverse_fb;
	int                       inverse_rl;
	int                       oldret;
};

struct
moveset
{
	bool                      (*allowed)(Move);
	bool                      (*allowed_next)(Move, Move, Move);
	Move                      sorted_moves[NMOVES+1];
	uint64_t                  mask[NMOVES][NMOVES];
};

struct
piecefilter
{
	bool                      epose;
	bool                      eposs;
	bool                      eposm;
	bool                      eofb;
	bool                      eorl;
	bool                      eoud;
	bool                      cp;
	bool                      coud;
	bool                      cofb;
	bool                      corl;
	bool                      cpos;
};

struct
prunedata
{
	char *                    filename;
	entry_group_t *           ptable;
	bool                      generated;
	uint64_t                  n;
	Coordinate *              coord;
	Moveset *                 moveset;
	bool                      compact;
	int                       base;
	uint64_t                  count[16];
	PruneData *               fallback;
	uint64_t                  fbmod;
};

struct
solveoptions
{
	int                       min_moves;
	int                       max_moves;
	int                       max_solutions;
	int                       nthreads;
	int                       optimal;
	bool                      can_niss;
	bool                      verbose;
	bool                      all;
	bool                      print_number;
	bool                      count_only;
};

struct
step
{
	char *                    shortname;
	char *                    name;
	bool                      final;
	Checker                   is_done;
	Estimator                 estimate;
	Checker                   ready;
	char *                    ready_msg;
	Validator                 is_valid;
	Moveset *                 moveset;
	Trans                     pre_trans;
	TransDetector             detect;
	int                       ntables;
	PruneData *               tables[10];
};

struct
symdata
{
	char *                    filename;
	bool                      generated;
	Coordinate *              coord;
	Coordinate *              sym_coord;
	int                       ntrans;
	Trans *                   trans;
	uint64_t *                class;
	uint64_t *                unsym;
	Trans *                   transtorep;
	uint64_t *                selfsim;
	CoordTransformer          transform; /* TODO: remove, use that of base coord */
};

struct
threaddatasolve
{
	int                       thid;
	Trans                     t;
	Cube                      cube;
	Step *                    step;
	int                       depth;
	SolveOptions *            opts;
	AlgList *                 start;
	AlgListNode **            node;
	AlgList *                 sols;
	pthread_mutex_t *         start_mutex;
	pthread_mutex_t *         sols_mutex;
};

struct
threaddatagenpt
{
	int                       thid;
	int                       nthreads;
	PruneData *               pd;
	int                       d;
	int                       nchunks;
	pthread_mutex_t **        mutex;
	pthread_mutex_t *         upmutex;
};

#endif
