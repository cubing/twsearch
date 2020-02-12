#ifndef GENERATINGSET_H
#include <vector>
#include "puzdef.h"
#include <iostream>
using namespace std ;
extern vector<int> movbuf ;
struct cons {
   cons(int m) {
      basemove = m ;
      inva = 0 ;
      a = b = 0 ;
      len = (basemove >= 0) ;
      leaf = 1 ;
   }
   static int invmove(const puzdef &pd, int mv) {
      int order = pd.basemoveorders[pd.moves[mv].base] ;
      int tw = pd.moves[mv].twist ;
      int ntw = (order - tw) % order ;
      return mv + ntw - tw ;
   }
   cons(const puzdef &pd, cons *a, cons *b, int inva=0) : a(a), b(b), inva(inva) {
      if (0 && a->leaf && a->basemove < 0) {
         *this = *b ;
      } else if (0 && b->leaf && b->basemove < 0) {
         *this = *a ;
         if (inva)
            basemove = invmove(pd, basemove) ;
      } else if (0 && a->leaf && b->leaf &&
                 pd.moves[a->basemove].base == pd.moves[b->basemove].base) {
         int tw = pd.moves[b->basemove].twist ;
         *this = *a ;
         if (inva)
            tw -= pd.moves[a->basemove].twist ;
         else
            tw += pd.moves[a->basemove].twist ;
         int order = pd.basemoveorders[pd.moves[a->basemove].base] ;
         tw = (tw + order) % order ;
         if (tw == 0) {
            basemove = -1 ;
            len = 0 ;
         } else {
            basemove = b->basemove + tw - pd.moves[b->basemove].twist ;
            len = 1 ;
         }
      } else {
         len = a->len + b->len ;
         basemove = -1 ;
         leaf = 0 ;
      }
   }
   void addmove(const puzdef &pd, int mv) {
      if (movbuf.size() == 0) {
         movbuf.push_back(mv) ;
      } else {
         int pmv = movbuf[movbuf.size()-1] ;
         if (pd.moves[mv].base == pd.moves[pmv].base) {
            movbuf.pop_back() ;
            int tw = pd.moves[mv].twist + pd.moves[pmv].twist ;
            int order = pd.basemoveorders[pd.moves[mv].base] ;
            tw = (tw + order) % order ;
            if (tw != 0)
               movbuf.push_back(mv+tw-pd.moves[mv].twist) ;
         } else {
            movbuf.push_back(mv) ;
         }
      }
   }
   void showmoves(const puzdef &pd, int inv) {
      movbuf.clear() ;
      showmovesr(pd, inv) ;
      cout << " " << movbuf.size() ;
//    for (auto mv : movbuf)
//       cout << " " << pd.moves[mv].name ;
   }
   void showmovesr(const puzdef &pd, int inv) {
      if (leaf) {
         if (basemove >= 0) {
            if (inv) {
               addmove(pd, invmove(pd, basemove)) ;
            } else {
               addmove(pd, basemove) ;
            }
         }
         return ;
      }
      if (inv) {
         b->showmovesr(pd, 1) ;
         a->showmovesr(pd, !inva) ;
      } else {
         a->showmovesr(pd, inva) ;
         b->showmovesr(pd, 0) ;
      }
   }
   long double len ;
   struct cons *a, *b ;
   int basemove ;         // -1 means no move
   char inva, leaf ;
} ;
struct generatingset {
   generatingset(const puzdef &pd) ;
   const puzdef &pd ;
   setval e ;
   vector<vector<setval>> sgs, sgsi, tk ;
   vector<vector<cons *>> tklen, len ;
   bool resolve(const setval p_) ;
   void knutha(int k1, int k2, const setval &p, cons *c) ;
   void knuthb(int k1, int k2, const setval &p, cons *c) ;
} ;
#define GENERATINGSET_H
#endif
