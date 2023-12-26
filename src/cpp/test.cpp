#include "test.h"
#include "cmds.h"
#include "generatingset.h"
#include "prunetable.h"
#include "solve.h"
#include "twsearch.h" // for checkbeforesolve
#include <iostream>
void timingtest(puzdef &pd) {
  stacksetval p1(pd), p2(pd);
  pd.assignpos(p1, pd.solved);
  const int NUMRAND = 65536;
  unsigned short randpool[NUMRAND];
  for (int i = 0; i < NUMRAND; i++)
    randpool[i] = myrand(pd.moves.size());
  cout << "Timing moves." << endl << flush;
  duration();
  int cnt = 100000000;
  for (int i = 0; i < cnt; i += 2) {
    int rmv = randpool[i & (NUMRAND - 1)];
    pd.mul(p1, pd.moves[rmv].pos, p2);
    rmv = randpool[1 + (i & (NUMRAND - 1))];
    pd.mul(p2, pd.moves[rmv].pos, p1);
  }
  double tim = duration();
  cout << "Did " << cnt << " in " << tim << " rate " << cnt / tim / 1e6 << endl
       << flush;
  cout << "Timing moves plus hash." << endl << flush;
  duration();
  cnt = 100000000;
  ull sum = 0;
  for (int i = 0; i < cnt; i += 2) {
    int rmv = randpool[i & (NUMRAND - 1)];
    pd.mul(p1, pd.moves[rmv].pos, p2);
    sum += fasthash(pd.totsize, p2);
    rmv = randpool[1 + (i & (NUMRAND - 1))];
    pd.mul(p2, pd.moves[rmv].pos, p1);
    sum += fasthash(pd.totsize, p1);
  }
  tim = duration();
  cout << "Did " << cnt << " in " << tim << " rate " << cnt / tim / 1e6
       << " sum " << sum << endl
       << flush;
  if ((int)pd.rotgroup.size() > 1) {
    cout << "Timing moves plus symmetry." << endl << flush;
    duration();
    cnt = 100000000;
    ull sum = 0;
    for (int i = 0; i < cnt; i++) {
      int rmv = randpool[i & (NUMRAND - 1)];
      pd.mul(p1, pd.moves[rmv].pos, p2);
      slowmodm2(pd, p2, p1);
      sum += fasthash(pd.totsize, p2);
    }
    tim = duration();
    cout << "Did " << cnt << " in " << tim << " rate " << cnt / tim / 1e6
         << " sum " << sum << endl
         << flush;
  }
  prunetable pt(pd, maxmem);
  duration();
  for (int tt = 0; tt < 2; tt++) {
    cout << "Timing moves plus lookup." << endl << flush;
    duration();
    cnt = 100000000;
    sum = 0;
    stacksetval looktmp(pd);
    for (int i = 0; i < cnt; i += 2) {
      int rmv = randpool[i & (NUMRAND - 1)];
      pd.mul(p1, pd.moves[rmv].pos, p2);
      sum += pt.lookup(p2, &looktmp);
      rmv = randpool[1 + (i & (NUMRAND - 1))];
      pd.mul(p2, pd.moves[rmv].pos, p1);
      sum += pt.lookup(p1, &looktmp);
    }
    tim = duration();
    cout << "Did " << cnt << " in " << tim << " rate " << cnt / tim / 1e6
         << " sum " << sum << endl
         << flush;
  }
  const int MAXLOOK = 8;
  ull tgo[MAXLOOK];
  for (int look = 2; look <= MAXLOOK; look *= 2) {
    int mask = look - 1;
    for (int i = 0; i < look; i++)
      tgo[i] = 0;
    cout << "Timing moves plus lookup piped " << look << endl << flush;
    duration();
    cnt = 100000000;
    sum = 0;
    if ((int)pd.rotgroup.size() > 1) {
      for (int i = 0; i < cnt; i++) {
        int rmv = randpool[i & (NUMRAND - 1)];
        pd.mul(p1, pd.moves[rmv].pos, p2);
        slowmodm2(pd, p2, p1);
        sum += pt.lookuphindexed(tgo[i & mask]);
        tgo[i & mask] = pt.indexhash(pd.totsize, p1);
        pt.prefetchindexed(tgo[i & mask]);
      }
    } else {
      for (int i = 0; i < cnt; i += 2) {
        int rmv = randpool[i & (NUMRAND - 1)];
        pd.mul(p1, pd.moves[rmv].pos, p2);
        sum += pt.lookuphindexed(tgo[i & mask]);
        tgo[i & mask] = pt.indexhash(pd.totsize, p2);
        pt.prefetchindexed(tgo[i & mask]);
        rmv = randpool[1 + (i & (NUMRAND - 1))];
        pd.mul(p2, pd.moves[rmv].pos, p1);
        sum += pt.lookuphindexed(tgo[1 + (i & mask)]);
        tgo[1 + (i & mask)] = pt.indexhash(pd.totsize, p1);
        pt.prefetchindexed(tgo[1 + (i & mask)]);
      }
    }
    tim = duration();
    cout << "Did " << cnt << " in " << tim << " rate " << cnt / tim / 1e6
         << " sum " << sum << endl
         << flush;
  }
}
void solvetest(puzdef &pd, int scramblemoves, generatingset *gs) {
  stacksetval p1(pd), p2(pd);
  pd.assignpos(p1, pd.solved);
  prunetable pt(pd, maxmem);
  while (1) {
    solve(pd, pt, p1, gs);
    for (ll i = 0; i < scramblemoves; i++) {
      while (1) {
        int rmv = myrand(pd.moves.size());
        pd.mul(p1, pd.moves[rmv].pos, p2);
        if (pd.legalstate(p2)) {
          pd.assignpos(p1, p2);
          break;
        }
      }
    }
  }
}
static struct testcmd : cmd {
  testcmd() : cmd("-T", "Run microbenchmark tests.") {}
  virtual void docommand(puzdef &pd) { timingtest(pd); }
} registertest;
static struct solvetestcmd : cmd {
  solvetestcmd()
      : cmd("-S",
            "Test solves by doing increasingly long random sequences.\n"
            "An integer argument can be provided appended to the S (as in -S5) "
            "to\n"
            "indicate the number of random moves to apply at each step.") {}
  virtual void docommand(puzdef &pd) { solvetest(pd, scramblemoves, gs); }
  void parse_args(int *, const char ***argv) {
    const char *p = **argv + 2;
    if (*p) {
      scramblemoves = atoll(p);
    } else {
      scramblemoves = 1;
    }
  }
  int scramblemoves;
} registersolvetest;
void ensure_test_is_linked() {}
