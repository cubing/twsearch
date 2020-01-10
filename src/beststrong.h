/*
 *   For small puzzles, attempts to find a best (in terms of short
 *   words) strong generating set by building a stabilizer chain
 *   for every ordering of bases (cubies).  It fills the table
 *
 *   sgs[fixedbm][position][cubie] = <seq>
 *
 *   for every possible value of fixedbm and position and cubie,
 *   where the cubie and the position differ.
 *
 *   For now we work in the identity supergroup for those puzzles
 *   that have identical pieces.
 *
 *   The way it works is we simply try every move sequence and use it
 *   to fill in what entries we can.  We compute those entries by
 *   determining what cubies are in the correct position, and
 *   running through the relevant table exhaustively.  This may be
 *   very time consuming.
 *
 *   Future extensions to make it actually *work*:
 *      - Search by commutators
 *      - Search by repeated execution of the sequences
 *        (this might be particularly effective)
 *      - Add a pruning table when we get the table mostly
 *        filled and have remaining entries (maybe by just
 *        enumerating the resulting positions and solving
 *        them with an optimal solver or something).
 *
 *   For now we do *not* attempt to combine entries.  This means
 *   that we may never actually fill any tables completely.
 *
 *   For a Rubik's cube the size of the table is 2^20 * 20 * 24 which
 *   is 503M---and that's just for the pointers.  About two thirds of
 *   these entries are not possible implicitly (because the position is
 *   in the solved set, or the value is in the solved set).
 *
 *   To start we allocate a vector just for pointers from the fixed
 *   point bitmap; from there we allocate a two-dimensional array
 *   that points to the actual sequences.
 */
#ifndef BESTSTRONG
#include <vector>
#include "puzdef.h"
using namespace std ;
using uchar = unsigned char ;
using seq = vector<uchar> ;
struct bestgeneratingset {
   bestgeneratingset(const puzdef &pd) ;
   void fillrecur(int togo, int sp, int st) ;
   const puzdef &pd ;
   seq ***bm ;
   int bmbits, numpos, hival ;
   seq solseq ;
   ll filled, sols, bms, slots ;
} ;
#define BESTSTRONG
#endif
