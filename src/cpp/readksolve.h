#ifndef READKSOLVE_H
#include "puzdef.h"
#include <cstdio>
#include <set>
/*
 *   This code manages parsing a tws file (an extension of the ksolve
 *   format).
 */
vector<string> getline(istream *f, ull &checksum);
void expect(const vector<string> &toks, int cnt);
int getnumber(int minval, const string &s);
allocsetval readposition(puzdef &pz, char typ, istream *f, ull &checksum,
                         bool zero_indexed);
puzdef readdef(istream *f);
void addmovepowers(puzdef &pd);
extern int nocorners, nocenters, noedges, ignoreori, distinguishall;
extern set<string> omitsets, omitperms, omitoris;
#define READKSOLVE_H
#endif
