#ifndef PRUNETABLE_H
#include "puzdef.h"
#include "threads.h"
#include "workchunks.h"
#include "rotations.h"
#include "canon.h"
#include <iostream>
/*
 *   This code supports pruning tables for arbitrary puzzles.  Memory
 *   consumption is a power of two.  Each table entry is two bits.
 *   There is an in-cache-line sub-table with four bits.  So each
 *   64-byte cache line has a 4-bit subentry and 510 "regular" entries.
 *
 *   For the two-bit entries, 0 means look at the subentry, 1 means the
 *   value is the current base, 2 means the value is base+1, and 3
 *   means the value is at least base+2.  The base changes as the
 *   pruning table is filled more, level by level.
 *
 *   The hash table permits collisions (stores the minimum).
 */
const int CACHELINESIZE = 64 ;
// BLOCKSIZE bits has one secondary entry and as many primary ones as fit.
// All of these values are log2 values.  SECONDARYBITS should be greater than
// PRIMARYBITS.
// Given a hash value h that's been adjusted to fit in the range, we
// want to make sure the low bit is 0 if PRIMARYBITS is 1.  Also, of
// the low (BLOCKSIZEBITS-PRIMARYBITS), we want to make sure the value
// is at least PRIMARYBITS.
// different way to ensure we set the low bits:  pick off the low 16,
// subtract a shifted fraction of them, then add the new base.
#define BLOCKSIZEBITS (6)
#define PRIMARYBITS (1)
#define SECONDARYBITS (2)
//
#define PRIMARYMASK ((1<<(1<<PRIMARYBITS))-1)
#define SECONDARYMASK ((1<<(1<<SECONDARYBITS))-1)
#define HSHIFT (6 - PRIMARYBITS)
#define HMUL (1+PRIMARYBITS)
#define LOWHMASK ((1<<HSHIFT)-1)
#define LOWHMIN (1<<(SECONDARYBITS-PRIMARYBITS)) ;
#define LOWCHECK ((1<<(BLOCKSIZEBITS-PRIMARYBITS)) - (1<<(SECONDARYBITS-PRIMARYBITS)))
#define LOWCLEARMASK ((1 << (BLOCKSIZEBITS-PRIMARYBITS)) - 1)
//
const int COMPSIGNATURE = 23 ; // start and end of data files
const int UNCOMPSIGNATURE = 24 ; // start and end of data files
#define UNKNOWNPUZZLE "unknownpuzzle"
extern string inputbasename ;
extern int nowrite ;
extern int startprunedepth ;
ull fasthash(int n, const setval sv) ;
struct prunetable ;
struct workerparam {
   workerparam(const puzdef &pd_, prunetable &pt_, int tid_) :
      pd(pd_), pt(pt_), tid(tid_) {}
   const puzdef &pd ;
   prunetable &pt ;
   int tid ;
} ;
extern vector<workerparam> workerparams ;
int setupthreads(const puzdef &pd, prunetable &pt) ;
const int BLOCKSIZE = 32768 ; // in long longs
const int FILLCHUNKS = 256 ; // long longs
struct fillbuf {
   int nchunks ;
   ull chunks[FILLCHUNKS] ;
} ;
struct fillworker {
   vector<allocsetval> posns ;
   int d ;
   fillbuf fillbufs[MEMSHARDS] ;
   allocsetval *looktmp ;
   char pad[256] ;
   void init(const puzdef &pd, int d_) ;
   ull fillstart(const puzdef &pd, prunetable &pt, int w) ;
   ull fillflush(prunetable &pt, int shard) ;
   void dowork(const puzdef &pd, prunetable &pt) ;
   ull filltable(const puzdef &pd, prunetable &pt, int togo, int sp, int st) ;
} ;
extern fillworker fillworkers[MAXTHREADS] ;
struct ioworkitem {
   char state ;
   ull *mem ;
   ull longcnt ;
   uchar *buf ;
   prunetable *pt ;
   unsigned int bytecnt ;
} ;
struct ioqueue {
   void initin(struct prunetable *pt_, istream *f_ = 0) ;
   void initout(struct prunetable *pt_, ostream *f_ = 0) ;
   void waitthread(int i) ;
   void queuepackwork(ull *mem, ull longcnt,
                        uchar *buf, unsigned int bytecnt) ;
   void queueunpackwork(ull *mem, ull longcnt,
                        uchar *buf, unsigned int bytecnt) ;
   void finishall() ;
   int nextthread ;
   struct prunetable *pt ;
   ioworkitem ioworkitems[MAXTHREADS] ;
   istream *inf ;
   ostream *outf ;
} ;
extern struct ioqueue ioqueue ;
struct prunetable {
   prunetable() {}
   prunetable(const puzdef &pd, ull maxmem) ;
   void filltable(const puzdef &pd, int d) ;
   void checkextend(const puzdef &pd, int ignorelookups=0) ;
   int lookuph(ull h) const {
      h = indexhash(h) ;
      int v = PRIMARYMASK & (mem[h >> HSHIFT] >> ((h & LOWHMASK) * HMUL)) ;
      if (v == PRIMARYMASK) {
         h &= ~LOWCLEARMASK ;
         return (mem[h >> HSHIFT] & SECONDARYMASK) - 1 ;
      } else
#if PRIMARYBITS == 1
         return 2 - v + baseval ;
#else
         return 2 + baseval ;
#endif
   }
   void prefetch(ull h) const {
      __builtin_prefetch(mem+((indexhash(h)) >> HSHIFT)) ;
   }
   ull indexhash(ull lowb) const {
      ull h = lowb ;
      h -= h >> subshift ;
      const ull BS = BLOCKSIZEBITS - SECONDARYBITS ;
      const ull BP = BLOCKSIZEBITS - PRIMARYBITS ;
      h = (h - ((h & ((1LL << (memshift + BP)) - 1)) >> BS) -
                  (h & 1) + (1LL << (memshift + SECONDARYBITS - PRIMARYBITS))) >> memshift ;
      if ((h & LOWCHECK) == 0) {
          std::cout << "Fail with h " << std::hex << h << std::dec << std::endl << std::flush ;
          h = lowb ;
          std::cout << "Started with " << std::hex << lowb << std::dec << std::endl << std::flush ;
          h -= h >> subshift ;
          std::cout << "Next was " << std::hex << h << std::dec << std::endl << std::flush ;
          std::cout << "Sub1 " << std::hex << 
((h & ((1LL << (memshift + BP)) - 1)) >> BS)
<< std::dec << std::endl << std::flush ;
          std::cout << "Sub2 " << std::hex << 
(h&1)
<< std::dec << std::endl << std::flush ;
          std::cout << "Add " << std::hex << 
(1LL << (memshift + SECONDARYBITS - PRIMARYBITS))
<< std::dec << std::endl << std::flush ;
          exit(10) ;
      }
// std::cout << "ih " << std::hex << h << std::dec << std::endl ;
      return h ;
   }
   ull indexhash(int n, const setval sv) const {
      return indexhash(fasthash(n, sv)) ;
   }
   int lookup(const setval sv, setval *looktmp) const {
      ull h ;
      if ((int)pdp->rotgroup.size() > 1) {
         slowmodm2(*pdp, sv, *looktmp) ;
         h = indexhash(totsize, *looktmp) ;
      } else {
         h = indexhash(totsize, sv) ;
      }
      int v = PRIMARYMASK & (mem[h >> HSHIFT] >> ((h & LOWHMASK) * HMUL)) ;
      if (v == PRIMARYMASK) {
         h &= ~LOWCLEARMASK ;
         return (mem[h >> HSHIFT] & SECONDARYMASK) - 1 ;
      } else
#if PRIMARYBITS == 1
         return 2 - v + baseval ;
#else
         return 2 + baseval ;
#endif
   }
   void addlookups(ull lookups) {
      lookupcnt += lookups ;
   }
   ~prunetable() {
      if (mem) {
         free(mem) ;
         mem = 0 ;
      }
      for (int i=0; i<7; i++)
         if (tabs[i]) {
            free(tabs[i]) ;
            tabs[i] = 0 ;
         }
   }
   // if someone set options that affect the hash, we add a suffix to the
   // data file name to reflect this.
   void addsumdat(const puzdef &pd, string &filename) const ;
   string makefilename(const puzdef &pd) const ;
   ull calcblocksize(ull *mem, ull longcnt) ;
   void packblock(ull *mem, ull longcnt, uchar *buf, ull bytecnt) ;
   void unpackblock(ull *mem, ull longcnt, uchar *block, int bytecnt) ;
   void writeblock(ull *mem, ull longcnt) ;
   void readblock(ull *mem, ull explongcnt, istream *f) ;
   void writept(const puzdef &pd) ;
   int readpt(const puzdef &pd) ;
   const puzdef *pdp ;
   ull size, popped, totpop, ptotpop ;
   ull subshift, memshift ;
   ull lookupcnt ;
   ull fillcnt ;
   ull *mem ;
   int totsize ;
   int shardshift ;
   int baseval, hibase ; // 0 is less; 1 is this; 2 is this+1; 3 is >=this+2
   int wval, wbval ;
   uchar codewidths[544] ;
   ull codevals[544] ;
   short *tabs[7] ;
   char justread ;
} ;
#define PRUNETABLE_H
#endif
