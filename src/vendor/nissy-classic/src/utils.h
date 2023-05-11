#ifndef UTILS_H
#define UTILS_H

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define POW2TO6             64ULL
#define POW2TO11            2048ULL
#define POW2TO12            4096ULL
#define POW3TO7             2187ULL
#define POW3TO8             6561ULL
#define FACTORIAL4          24ULL
#define FACTORIAL6          720ULL
#define FACTORIAL7          5040ULL
#define FACTORIAL8          40320ULL
#define FACTORIAL12         479001600ULL
#define BINOM12ON4          495ULL
#define BINOM8ON4           70ULL
#define MIN(a,b)            (((a) < (b)) ? (a) : (b))
#define MAX(a,b)            (((a) > (b)) ? (a) : (b))

void        apply_permutation(int *perm, int *set, int n);
int         binomial(int n, int k);
int         digit_array_to_int(int *a, int n, int b);
int         factorial(int n);
void        index_to_perm(int p, int n, int *r);
void        index_to_subset(int s, int n, int k, int *r);
void        int_to_digit_array(int a, int b, int n, int *r);
void        int_to_sum_zero_array(int x, int b, int n, int *a);
int         invert_digits(int a, int b, int n);
bool        is_perm(int *a, int n);
bool        is_subset(int *a, int n, int k);
int         perm_sign(int *a, int n);
int         perm_to_index(int *a, int n);
int         powint(int a, int b);
int         subset_to_index(int *a, int n, int k);
void        sum_arrays_mod(int *src, int *dst, int n, int m);
void        swap(int *a, int *b);
void        swapu64(uint64_t *a, uint64_t *b);

#endif
