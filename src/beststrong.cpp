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
#include "beststrong.h"
#include "canon.h"
#include <iostream>
void bestgeneratingset::fillrecur(int togo, int sp, int st) {
   if (togo == 0) {
      ull sbm = 0 ;
      uchar *s = posns[sp].dat ;
      int at = 0 ;
      int bmat = 0 ;
      for (int i=0; i<(int)pd.setdefs.size(); i++) {
         const setdef &sd = pd.setdefs[i] ;
         int ssz = sd.size ;
         for (int j=0; j<ssz; j++, bmat++, at++)
            if (s[at] == pd.solved.dat[at] && s[at+ssz] == pd.solved.dat[at+ssz])
               sbm |= 1LL << bmat ;
         at += ssz ;
      }
      ull tbm = 0 ;
      seq *sol = 0 ;
      do {
         if (bm[tbm] == 0) {
            bms++ ;
            bm[tbm] = (seq **)calloc(bmbits*hival, sizeof(*bm[tbm])) ;
         }
         at = 0 ;
         seq **dat = bm[tbm] ;
         for (int i=0; i<(int)pd.setdefs.size(); i++) {
            const setdef &sd = pd.setdefs[i] ;
            for (int j=0; j<(int)sd.size; j++, at++) {
               if ((sbm >> at) & 1)
                  continue ;
               int val = s[at] * sd.omod + s[at+sd.size] ;
               int off = at * hival + val ;
               if (dat[off] == 0) {
                  if (sol == 0) {
//                   cout << hex << sbm << dec ;
//                   for (int k=0; k<pd.totsize; k++)
//                      cout << " " << (int)s[k] ;
//                   cout << endl ;
                     sols++ ;
                     sol = new seq(solseq) ;
                  }
                  dat[off] = sol ;
                  filled++ ;
                  slots++ ;
                  if ((slots & (slots - 1)) == 0) {
                     cout << " " << slots << flush ;
                  }
               }
            }
            at += sd.size ;
         }
         tbm=((tbm | ~sbm) + 1) & sbm ;
      } while (tbm != 0) ;
      return ;
   }
   ull mask = canonmask[st] ;
   const vector<int> &ns = canonnext[st] ;
   for (int m=0; m<(int)pd.moves.size(); m++) {
      const moove &mv = pd.moves[m] ;
      if ((mask >> mv.cs) & 1)
         continue ;
      pd.mul(posns[sp], mv.pos, posns[sp+1]) ;
      if (!pd.legalstate(posns[sp+1]))
         continue ;
      solseq[sp] = m ;
      fillrecur(togo-1, sp+1, ns[mv.cs]) ;
   }
}
bestgeneratingset::bestgeneratingset(const puzdef &pd) : pd(pd) {
   bmbits = pd.totsize >> 1 ;
   if (bmbits > 32)
      error("! too many pieces to do this.") ;
   hival = 0 ;
   for (int i=0; i<(int)pd.setdefs.size(); i++) {
      const setdef &sd = pd.setdefs[i] ;
      int tsz = sd.size * sd.omod ;
      if (tsz > hival)
         hival = tsz ;
   }
   bm = (seq ***)calloc(1LL<<bmbits, sizeof(bm[0])) ;
   filled = 0 ;
   sols = 0 ;
   bms = 0 ;
   slots = 0 ;
   for (int d=1; ; d++) {
      slots = 0 ;
      cout << "Doing depth " << d << endl << flush ;
      while (posns.size() <= 100 || (int)posns.size() <= d+1)
         posns.push_back(allocsetval(pd, pd.solved)) ;
      pd.assignpos(posns[0], pd.solved) ;
      solseq.resize(d) ;
      fillrecur(d, 0, 0) ;
      cout << endl ;
      cout << "Filled is " << filled << " sols " << sols << " bms " << bms << " in " << duration() << endl << flush; 
   }
}
