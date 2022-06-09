#include <map>
#include <iostream>
#include "canon.h"
#include "ordertree.h"
#include "findalgo.h"
static map<ll, ll> cnts ;
ll tot = 0 ;
vector<ll> best ;
void recurorder(const puzdef &pd, int togo, int sp, int st, int fm) {
   if (togo == 0) {
      vector<int> cc = pd.cyccnts(posns[sp]) ;
/*
 for (int i=1; i<(int)cc.size(); i++)
   if (cc[i])
      cout << " " << i << ":" << cc[i] ;
 cout << endl ;
 */
      ll ps = 0 ;
      for (int i=1; i<(int)cc.size(); i++)
         ps += i * cc[i] ;
      if (best.size() == 0) {
         best.resize(ps+1, -1) ;
      }
      vector<pair<int, int>> primes ;
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
      vector<ll> fa ;
      fa.push_back(1) ;
      for (auto pp: primes) {
         ll nsz = fa.size() * pp.second ;
         for (ll i=0; i<nsz; i++)
            fa.push_back(fa[i]*pp.first) ;
      }
      for (auto f: fa) {
         int fixed = cc[1] ;
         for (ll i=2; i<(int)cc.size(); i++)
            if (cc[i] != 0 && f % i == 0)
               fixed += i * cc[i] ;
         if (fixed != 0 && ps != fixed) {
            if (best[ps-fixed] < 0 || best[ps-fixed] >= sp * f) {
               best[ps-fixed] = sp * f ;
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
   for (int d=1; ; d++) {
      posns.clear() ;
      movehist.clear() ;
      while ((int)posns.size() <= d + 1) {
         posns.push_back(allocsetval(pd, pd.id)) ;
         movehist.push_back(-1) ;
      }
      recurorder(pd, d, 0, 0, -1) ;
   }
}
