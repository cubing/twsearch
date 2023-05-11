#include "pf.h"

PieceFilter
pf_all = {
	.epose = true,
	.eposs = true,
	.eposm = true,
	.eofb  = true,
	.eorl  = true,
	.eoud  = true,
	.cp    = true,
	.cofb  = true,
	.corl  = true,
	.coud  = true,
	.cpos  = true
};

PieceFilter
pf_4val = {
	.epose = true,
	.eposs = true,
	.eposm = true,
	.eofb  = true,
	.coud  = true,
	.cp    = true
};

PieceFilter
pf_epcp = {
	.epose = true,
	.eposs = true,
	.eposm = true,
	.cp    = true
};

PieceFilter
pf_cpos = {
	.cpos  = true
};

PieceFilter
pf_cp = {
	.cp    = true
};

PieceFilter
pf_ep = {
	.epose = true,
	.eposs = true,
	.eposm = true
};

PieceFilter
pf_e = {
	.epose = true
};

PieceFilter
pf_s = {
	.eposs = true
};

PieceFilter
pf_m = {
	.eposm = true
};

PieceFilter
pf_eo = {
	.eofb  = true,
	.eorl  = true,
	.eoud  = true
};

PieceFilter
pf_co = {
	.cofb  = true,
	.corl  = true,
	.coud  = true
};

PieceFilter
pf_coud = {
	.coud  = true
};

PieceFilter
pf_edges = {
	.epose = true,
	.eposs = true,
	.eposm = true,
	.eofb  = true,
	.eorl  = true,
	.eoud  = true
};
