#include "utils.h"

void
apply_permutation(int *perm, int *set, int n)
{
	int *aux = malloc(n * sizeof(int));
	int i;

	if (!is_perm(perm, n))
		return;
		
	for (i = 0; i < n; i++)
		aux[i] = set[perm[i]];

	memcpy(set, aux, n * sizeof(int));
	free(aux);
}

int
binomial(int n, int k)
{
	if (n < 0 || k < 0 || k > n)
		return 0;

	return factorial(n) / (factorial(k) * factorial(n-k));
}

int
digit_array_to_int(int *a, int n, int b)
{
	int i, ret = 0, p = 1;

	for (i = 0; i < n; i++, p *= b)
		ret += a[i] * p;

	return ret;
}

int
factorial(int n)
{
	int i, ret = 1;

	if (n < 0)
		return 0;

	for (i = 1; i <= n; i++)
		ret *= i;

	return ret;
}

void
index_to_perm(int p, int n, int *r)
{
	int *a = malloc(n * sizeof(int));
	int i, j, c;

	for (i = 0; i < n; i++)
		a[i] = 0;

	if (p < 0 || p >= factorial(n))
		for (i = 0; i < n; i++)
			r[i] = -1;

	for (i = 0; i < n; i++) {
		c = 0;
		j = 0;
		while (c <= p / factorial(n-i-1))
			c += a[j++] ? 0 : 1;
		r[i] = j-1;
		a[j-1] = 1;
		p %= factorial(n-i-1);
	}

	free(a);
}

void
index_to_subset(int s, int n, int k, int *r)
{
	int i, j, v;

	if (s < 0 || s >= binomial(n, k)) {
		for (i = 0; i < n; i++)
			r[i] = -1;
		return;
	}

	for (i = 0; i < n; i++) {
		if (k == n-i) {
			for (j = i; j < n; j++)
				r[j] = 1;
			return;
		}

		if (k == 0) {
			for (j = i; j < n; j++)
				r[j] = 0;
			return;
		}

		v = binomial(n-i-1, k);
		if (s >= v) {
			r[i] = 1;
			k--;
			s -= v;
		} else {
			r[i] = 0;
		}
	}
}

void
int_to_digit_array(int a, int b, int n, int *r)
{
	int i;

	if (b <= 1)
		for (i = 0; i < n; i++)
			r[i] = 0;
	else
		for (i = 0; i < n; i++, a /= b)
			r[i] = a % b;
}

void
int_to_sum_zero_array(int x, int b, int n, int *a)
{
	int i, s = 0;

	if (b <= 1) {
		for (i = 0; i < n; i++)
		    a[i] = 0;
	} else {
		int_to_digit_array(x, b, n-1, a);
		for (i = 0; i < n - 1; i++)
		    s = (s + a[i]) % b;
		a[n-1] = (b - s) % b;
	}
}

int
invert_digits(int a, int b, int n)
{
	int i, ret, *r = malloc(n * sizeof(int));

	int_to_digit_array(a, b, n, r);
	for (i = 0; i < n; i++)
		r[i] = (b-r[i]) % b;

	ret = digit_array_to_int(r, n, b);
	free(r);
	return ret;
}

bool
is_perm(int *a, int n)
{
	int *aux = malloc(n * sizeof(int));
	int i;

	for (i = 0; i < n; i++)
		aux[i] = 0;
	
	for (i = 0; i < n; i++) {
		if (a[i] < 0 || a[i] >= n)
			return false;
		else
			aux[a[i]] = 1;
	}

	for (i = 0; i < n; i++)
		if (!aux[i])
			return false;

	free(aux);

	return true;
}

bool
is_subset(int *a, int n, int k)
{
	int i, sum = 0;

	for (i = 0; i < n; i++)
		sum += a[i] ? 1 : 0;

	return sum == k;
}

int
perm_sign(int *a, int n)
{
	int i, j, ret = 0;

	if (!is_perm(a,n))
		return -1;

	for (i = 0; i < n; i++)
		for (j = i+1; j < n; j++)
			ret += (a[i] > a[j]) ? 1 : 0;

	return ret % 2;
}

int
perm_to_index(int *a, int n)
{
	int i, j, c, ret = 0;

	if (!is_perm(a, n))
		return -1;

	for (i = 0; i < n; i++) {
		c = 0;
		for (j = i+1; j < n; j++)
			c += (a[i] > a[j]) ? 1 : 0;
		ret += factorial(n-i-1) * c;
	}

	return ret;
}

int
powint(int a, int b)
{
	if (b < 0)
		return 0;
	if (b == 0)
		return 1;

	if (b % 2)
		return a * powint(a, b-1);
	else
		return powint(a*a, b/2);
}

int 
subset_to_index(int *a, int n, int k)
{
	int i, ret = 0;

	if (!is_subset(a, n, k))
		return binomial(n, k);

	for (i = 0; i < n; i++) {
		if (k == n-i)
			return ret;
		if (a[i]) {
			ret += binomial(n-i-1, k);
			k--;
		}
	}

	return ret;
}

void
sum_arrays_mod(int *src, int *dst, int n, int m)
{
	int i;

	for (i = 0; i < n; i++)
		dst[i] = (m <= 0) ? 0 : (src[i] + dst[i]) % m;
}

void
swap(int *a, int *b)
{
	int aux;

	aux = *a;
	*a  = *b;
	*b  = aux;
}

void
swapu64(uint64_t *a, uint64_t *b)
{
	uint64_t aux;

	aux = *a;
	*a  = *b;
	*b  = aux;
}

