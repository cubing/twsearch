#include <map>
#include <iostream>
#include "canon.h"
#include "ordertree.h"
#include "findalgo.h"
#include "solve.h"
static map<ll, ll> cnts ;
ll tot = 0 ;
vector<ll> best ;
vector<ull> solfound ;
ll otcnt = 0 ;
static vector<pair<int, int>> primes ;
static vector<ll> fa ;
static vector<int> fac ;
static vector<int> cc ;
void recurorder(const puzdef &pd, int togo, int sp, int st, int fm) {
   if (togo == 0) {
      otcnt++ ;
      pd.cyccnts(cc, posns[sp]) ;
/*
 for (int i=1; i<(int)cc.size(); i++)
   if (cc[i])
      cout << " " << i << ":" << cc[i] ;
 cout << endl ;
 */
      ll ps = 0 ;
      for (int i=1; i<(int)cc.size(); i++)
         ps += i * cc[i] ;
      if ((int)best.size() <= ps) {
         best.resize(ps+1, -1) ;
         solfound.resize(ps+1, -1) ;
      }
      primes.clear() ;
      for (int p=2; p<(int)cc.size(); p++)
         if (isprime(p)) {
            int cnt = 0 ;
            for (int j=1, pp=p; pp<(int)cc.size(); pp *= p, j++)
               for (int i=pp; i<(int)cc.size(); i += pp)
                  if (cc[i] > 0) {
                     cnt = j ;
                  }
            if (cnt > 0) {
// cout << p << " " << cnt << endl ;
               primes.push_back({p, cnt}) ;
            }
         }
      fa.clear() ;
      fa.push_back(1) ;
      for (auto pp: primes) {
         ll nsz = fa.size() * pp.second ;
         for (ll i=0; i<nsz; i++)
            fa.push_back(fa[i]*pp.first) ;
      }
      fac.clear() ;
      fac.resize(fa.size()) ;
      for (int i=1; i<(int)cc.size(); i++)
         if (cc[i]) {
            int v = i * cc[i] ;
            for (int j=0; j<(int)fa.size(); j++)
               if (fa[j] % i == 0)
                  fac[j] += v ;
         }
      for (int j=0; j<(int)fa.size(); j++) {
         auto f = fa[j] ;
         int fixed = fac[j] ;
         if (fixed != 0 && ps != fixed) {
            if (best[ps-fixed] < 0 || best[ps-fixed] > sp * f ||
      (best[ps-fixed] == sp * f && solfound[ps-fixed] < solutionsneeded)) {
               if (sp * f != best[ps-fixed]) {
                  best[ps-fixed] = sp * f ;
                  solfound[ps-fixed] = 0 ;
               }
               solfound[ps-fixed]++ ;
               cout << ps-fixed << " " << sp * f << " " << f ;
               for (int i=0; i<sp; i++)
                  cout << " " << pd.moves[movehist[i]].name ;
               cout << endl << flush ;
            }
         }
      }
/*
      cout << sp << ":" ;
      for (int i=2; i<(int)cc.size(); i++)
         if (cc[i]) {
            if (cc[i] == 1) {
               cout << " " << i ;
            } else {
               cout << " " << i << "^" << cc[i] ;
            }
         }
      cout << endl << flush ;
 */
/*
      ll o = puzdef::order(cc) ;
      cnts[o]++ ;
      tot++ ;
      if (tot > 1000 && (tot & (tot - 1)) == 0) {
         cout << "At " << tot << endl ;
         for (auto v: cnts)
            cout << " " << v.first << " " << v.second << endl ;
         cout << flush ;
         cout << o ;
         for (int i=0; i<sp; i++)
            cout << " " << pd.moves[movehist[i]].name ;
         cout << endl << flush ;
      }
 */
      return ;
   }
   ull mask = canonmask[st] ;
   const vector<int> &ns = canonnext[st] ;
   int tfm ;
   if (fm == -1)
      tfm = 0 ;
   else if (togo == 1)
      tfm = movehist[fm] + 1 ;
   else
      tfm = movehist[fm] ;
   for (int m=tfm; m<(int)pd.moves.size(); m++) {
      int nfm = fm + 1 ;
      fm = -1 ;
      const moove &mv = pd.moves[m] ;
      if ((mask >> mv.cs) & 1)
         continue ;
      movehist[sp] = m ;
      pd.mul(posns[sp], mv.pos, posns[sp+1]) ;
      if (pd.legalstate(posns[sp+1]))
         recurorder(pd, togo-1, sp+1, ns[mv.cs], nfm) ;
   }
}
void ordertree(const puzdef &pd) {
   for (int d=max(optmindepth, 1); d<=maxdepth; d++) {
      otcnt = 0 ;
      posns.clear() ;
      movehist.clear() ;
      while ((int)posns.size() <= d + 1) {
         posns.push_back(allocsetval(pd, pd.id)) ;
         movehist.push_back(-1) ;
      }
      recurorder(pd, d, 0, 0, -1) ;
      cout << "Depth " << d << " examined " << otcnt << " sequences." << endl ;
   }
}
