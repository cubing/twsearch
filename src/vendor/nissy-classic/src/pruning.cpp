#include "pruning.h"

#define ENTRIES_PER_GROUP              (2*sizeof(entry_group_t))
#define ENTRIES_PER_GROUP_COMPACT      (4*sizeof(entry_group_t))

static int         findchunk(PruneData *pd, int nchunks, uint64_t i);
static void        genptable_bfs(PruneData *pd, int d, int nt, int nc);
static void        genptable_compress(PruneData *pd);
static void        genptable_fixnasty(PruneData *pd, int d, int nthreads);
static void        genptable_setbase(PruneData *pd);
static void *      instance_bfs(void *arg);
static void *      instance_fixnasty(void *arg);
static void        ptable_update(PruneData *pd, Cube cube, int m);
static void        ptable_update_index(PruneData *pd, uint64_t ind, int m);
static int         ptableval_index(PruneData *pd, uint64_t ind);
static bool        read_ptable_file(PruneData *pd);
static bool        write_ptable_file(PruneData *pd);

PruneData
pd_eofb_HTM = {
	.filename = "pt_eofb_HTM",
	.coord    = &coord_eofb,
	.moveset  = &moveset_HTM,
};

PruneData
pd_coud_HTM = {
	.filename = "pt_coud_HTM",
	.coord    = &coord_coud,
	.moveset  = &moveset_HTM,
};

PruneData
pd_cornershtr_HTM = {
	.filename = "pt_cornershtr_HTM",
	.coord    = &coord_cornershtr,
	.moveset  = &moveset_HTM,
};

PruneData
pd_corners_HTM = {
	.filename = "pt_corners_HTM",
	.coord    = &coord_corners,
	.moveset  = &moveset_HTM,
};

PruneData
pd_drud_sym16_HTM = {
	.filename = "pt_drud_sym16_HTM",
	.coord    = &coord_drud_sym16,
	.moveset  = &moveset_HTM,
};

PruneData
pd_drud_eofb = {
	.filename = "pt_drud_eofb",
	.coord    = &coord_drud_eofb,
	.moveset  = &moveset_eofb,
};

PruneData
pd_drudfin_noE_sym16_drud = {
	.filename = "pt_drudfin_noE_sym16_drud",
	.coord    = &coord_drudfin_noE_sym16,
	.moveset  = &moveset_drud,
};

PruneData
pd_htr_drud = {
	.filename = "pt_htr_drud",
	.coord    = &coord_htr_drud,
	.moveset  = &moveset_drud,
};

PruneData
pd_cp_drud = {
	.filename = "pt_cp_drud",
	.coord    = &coord_cp,
	.moveset  = &moveset_drud,
};

PruneData
pd_htrfin_htr = {
	.filename = "pt_htrfin_htr",
	.coord    = &coord_htrfin,
	.moveset  = &moveset_htr,
};

PruneData
pd_nxopt31_HTM = {
	.filename = "pt_nxopt31_HTM",
	.coord    = &coord_nxopt31,
	.moveset  = &moveset_HTM,

	.compact  = true,
	.fallback = &pd_drud_sym16_HTM,
	.fbmod    = BINOM8ON4,
};

PruneData * all_pd[] = {
	&pd_eofb_HTM,
	&pd_coud_HTM,
	&pd_cornershtr_HTM,
	&pd_corners_HTM,
	&pd_drud_sym16_HTM,
	&pd_drud_eofb,
	&pd_drudfin_noE_sym16_drud,
	&pd_htr_drud,
	&pd_cp_drud,
	&pd_htrfin_htr,
	&pd_nxopt31_HTM,
	NULL
};

/* Functions *****************************************************************/

int
findchunk(PruneData *pd, int nchunks, uint64_t i)
{
	uint64_t chunksize;

	chunksize = pd->coord->max / (uint64_t)nchunks;
	chunksize += ENTRIES_PER_GROUP - (chunksize % ENTRIES_PER_GROUP);

	return MIN(nchunks-1, (int)(i / chunksize));
}

void
free_pd(PruneData *pd)
{
	if (pd->generated)
		free(pd->ptable);

	pd->generated = false;
}

void
genptable(PruneData *pd, int nthreads)
{
	bool compact;
	int d, nchunks;
	uint64_t oldn, sz;

	if (pd->generated)
		return;

	/* TODO: check if memory is enough, otherwise maybe exit gracefully? */
	sz = ptablesize(pd) * (pd->compact ? 2 : 1);
	pd->ptable = malloc(sz * sizeof(entry_group_t));

	if (read_ptable_file(pd)) {
		pd->generated = true;
		return;
	}

	if (nthreads < 4) {
		fprintf(stderr,
			"--- Warning ---\n"
			"You are using only %d threads to generate the pruning"
			"tables. This can take a while."
			"Unless you did this intentionally, you should re-run"
			"this command with `-t 4' or more.\n"
			"---------------\n\n", nthreads
		);
	}


	/* For the first steps we proceed the same way for compact and not */
	compact = pd->compact;
	pd->compact = false;
	pd->generated = true;

	nchunks = MIN(ptablesize(pd), 100000);
	fprintf(stderr, "Cannot load %s, generating it with %d threads\n",
			pd->filename, nthreads); 


	memset(pd->ptable, ~(uint8_t)0, ptablesize(pd)*sizeof(entry_group_t));

	ptable_update(pd, (Cube){0}, 0);
	pd->n = 1;
	oldn = 0;
	genptable_fixnasty(pd, 0, nthreads);
	fprintf(stderr, "Depth %d done, generated %"
		PRIu64 "\t(%" PRIu64 "/%" PRIu64 ")\n",
		0, pd->n - oldn, pd->n, pd->coord->max);
	oldn = pd->n;
	pd->count[0] = pd->n;
	for (d = 0; d < 15 && pd->n < pd->coord->max; d++) {
		genptable_bfs(pd, d, nthreads, nchunks);
		genptable_fixnasty(pd, d+1, nthreads);
		fprintf(stderr, "Depth %d done, generated %"
			PRIu64 "\t(%" PRIu64 "/%" PRIu64 ")\n",
			d+1, pd->n - oldn, pd->n, pd->coord->max);
		pd->count[d+1] = pd->n - oldn;
		oldn = pd->n;
	}
	fprintf(stderr, "Pruning table generated!\n");
	
	genptable_setbase(pd);
	if (compact)
		genptable_compress(pd);
	
	if (!write_ptable_file(pd))
		fprintf(stderr, "Error writing ptable file\n");
}

static void
genptable_bfs(PruneData *pd, int d, int nthreads, int nchunks)
{
	int i;
	pthread_t t[nthreads];
	ThreadDataGenpt td[nthreads];
	pthread_mutex_t *mtx[nchunks], *upmtx;

	upmtx = malloc(sizeof(pthread_mutex_t));
	pthread_mutex_init(upmtx, NULL);
	for (i = 0; i < nchunks; i++) {
		mtx[i] = malloc(sizeof(pthread_mutex_t));
		pthread_mutex_init(mtx[i], NULL);
	}

	for (i = 0; i < nthreads; i++) {
		td[i].thid     = i;
		td[i].nthreads = nthreads;
		td[i].pd       = pd;
		td[i].d        = d;
		td[i].nchunks  = nchunks;
		td[i].mutex    = mtx;
		td[i].upmutex  = upmtx;
		pthread_create(&t[i], NULL, instance_bfs, &td[i]);
	}

	for (i = 0; i < nthreads; i++)
		pthread_join(t[i], NULL);

	free(upmtx);
	for (i = 0; i < nchunks; i++)
		free(mtx[i]);
}

static void
genptable_compress(PruneData *pd)
{
	int val;
	uint64_t i, j;
	entry_group_t mask, v;

	fprintf(stderr, "Compressing table to 2 bits per entry\n");

	for (i = 0; i < pd->coord->max; i += ENTRIES_PER_GROUP_COMPACT) {
		mask = (entry_group_t)0;
		for (j = 0; j < ENTRIES_PER_GROUP_COMPACT; j++) {
			if (i+j >= pd->coord->max)
				break;
			val = ptableval_index(pd, i+j) - pd->base;
			v = (entry_group_t)MIN(3, MAX(0, val));
			mask |= v << (2*j);
		}
		pd->ptable[i/ENTRIES_PER_GROUP_COMPACT] = mask;
	}

	pd->compact = true;
	pd->ptable = realloc(pd->ptable, sizeof(entry_group_t)*ptablesize(pd));
}

static void
genptable_fixnasty(PruneData *pd, int d, int nthreads)
{
	int i;
	pthread_t t[nthreads];
	ThreadDataGenpt td[nthreads];
	pthread_mutex_t *upmtx;

	if (pd->coord->tfind == NULL)
		return;

	upmtx = malloc(sizeof(pthread_mutex_t));
	pthread_mutex_init(upmtx, NULL);
	for (i = 0; i < nthreads; i++) {
		td[i].thid     = i;
		td[i].nthreads = nthreads;
		td[i].pd       = pd;
		td[i].d        = d;
		td[i].upmutex  = upmtx;
		pthread_create(&t[i], NULL, instance_fixnasty, &td[i]);
	}

	for (i = 0; i < nthreads; i++)
		pthread_join(t[i], NULL);

	free(upmtx);
}

static void
genptable_setbase(PruneData *pd)
{
	int i;
	uint64_t sum, newsum;

	pd->base = 0;
	sum = pd->count[0] + pd->count[1] + pd->count[2];
	for (i = 3; i < 16; i++) {
		newsum = sum + pd->count[i] - pd->count[i-3];
		if (newsum > sum)
			pd->base = i-3;
		sum = newsum;
	}
}

static void *
instance_bfs(void *arg)
{
	ThreadDataGenpt *td;
	uint64_t i, ii, blocksize, rmin, rmax, updated;
	int j, pval, ichunk;
	Move *ms;

	td = (ThreadDataGenpt *)arg;
	ms = td->pd->moveset->sorted_moves;
	blocksize = td->pd->coord->max / (uint64_t)td->nthreads;
	rmin = ((uint64_t)td->thid) * blocksize;
	rmax = td->thid == td->nthreads - 1 ?
	       td->pd->coord->max :
	       ((uint64_t)td->thid + 1) * blocksize;

	updated = 0;
	for (i = rmin; i < rmax; i++) {
		ichunk = findchunk(td->pd, td->nchunks, i);
		pthread_mutex_lock(td->mutex[ichunk]);
		pval = ptableval_index(td->pd, i);
		pthread_mutex_unlock(td->mutex[ichunk]);
		if (pval == td->d) {
			for (j = 0; ms[j] != NULLMOVE; j++) {
				ii = td->pd->coord->move(ms[j], i);
				ichunk = findchunk(td->pd, td->nchunks, ii);
				pthread_mutex_lock(td->mutex[ichunk]);
				pval = ptableval_index(td->pd, ii);
				if (pval > td->d+1) {
					ptable_update_index(td->pd,
					    ii, td->d+1);
					updated++;
				}
				pthread_mutex_unlock(td->mutex[ichunk]);
			}
		}
	}

	pthread_mutex_lock(td->upmutex);
	td->pd->n += updated;
	pthread_mutex_unlock(td->upmutex);

	return NULL;
}

static void *
instance_fixnasty(void *arg)
{
	ThreadDataGenpt *td;
	uint64_t i, ii, nb, blocksize, rmin, rmax, updated;
	int j, n;
	Trans t, aux[NTRANS];

	td = (ThreadDataGenpt *)arg;
	nb = td->pd->coord->max / td->pd->coord->base->max;
	blocksize = (td->pd->coord->base->max / td->nthreads) * nb;
	rmin = ((uint64_t)td->thid) * blocksize;
	rmax = td->thid == td->nthreads - 1 ?
	       td->pd->coord->max :
	       ((uint64_t)td->thid + 1) * blocksize;

	updated = 0;
	for (i = rmin; i < rmax; i++) {
		if (ptableval_index(td->pd, i) == td->d) {
			if ((n = td->pd->coord->tfind(i, aux)) == 1)
				continue;

			for (j = 0; j < n; j++) {
				if ((t = aux[j]) == uf)
					continue;
				ii = td->pd->coord->transform(t, i);
				if (ptableval_index(td->pd, ii) > td->d) {
					ptable_update_index(td->pd, ii, td->d);
					updated++;
				}
			}
		}
	}

	pthread_mutex_lock(td->upmutex);
	td->pd->n += updated;
	pthread_mutex_unlock(td->upmutex);

	return NULL;
}

void
print_ptable(PruneData *pd)
{
	uint64_t i;

	if (!pd->generated)
		genptable(pd, 1); /* TODO: set default nthreads somewhere */
		
	printf("Table %s\n", pd->filename);
	printf("Base value: %d\n", pd->base);
	for (i = 0; i < 16; i++)
		printf("%2" PRIu64 "\t%10" PRIu64 "\n", i, pd->count[i]);
}

uint64_t
ptablesize(PruneData *pd)
{
	uint64_t e;

	e = pd->compact ? ENTRIES_PER_GROUP_COMPACT : ENTRIES_PER_GROUP;

	return (pd->coord->max + e - 1) / e;
}

static void
ptable_update(PruneData *pd, Cube cube, int n)
{
	ptable_update_index(pd, pd->coord->index(cube), n);
}

static void
ptable_update_index(PruneData *pd, uint64_t ind, int n)
{
	int sh;
	entry_group_t mask;
	uint64_t i;

	sh = 4 * (ind % ENTRIES_PER_GROUP);
	mask = ((entry_group_t)15) << sh;
	i = ind/ENTRIES_PER_GROUP;

	pd->ptable[i] &= ~mask;
	pd->ptable[i] |= (((entry_group_t)n)&15) << sh;
}

int
ptableval(PruneData *pd, Cube cube)
{
	return ptableval_index(pd, pd->coord->index(cube));
}

static int
ptableval_index(PruneData *pd, uint64_t ind)
{
	int sh, ret;
	entry_group_t mask;
	uint64_t i, e;
	entry_group_t m;

	if (!pd->generated) {
		fprintf(stderr, "Warning: request pruning table value"
			" for uninitialized table %s.\n It's fine, but it"
			" should not happen. Please report bug.\n",
			pd->filename);
		genptable(pd, 1); /* TODO: set default or remove this case */
	}

	if (pd->compact) {
		e  = ENTRIES_PER_GROUP_COMPACT;
		m  = 3;
		sh = (ind % e) * 2;
	} else {
		e  = ENTRIES_PER_GROUP;
		m  = 15;
		sh = (ind % e) * 4;
	}

	mask = m << sh;
	i = ind/e;

	ret = (pd->ptable[i] & mask) >> sh;

	if (pd->compact) {
		if (ret)
			ret += pd->base;
		else
			ret = ptableval_index(pd->fallback, ind / pd->fbmod);
	}

	return ret;
}

static bool
read_ptable_file(PruneData *pd)
{
	init_env();

	FILE *f;
	char fname[strlen(tabledir)+100];
	int i;
	uint64_t r;

	strcpy(fname, tabledir);
	strcat(fname, "/");
	strcat(fname, pd->filename);

	if ((f = fopen(fname, "rb")) == NULL)
		return false;

	r = fread(&(pd->base), sizeof(int), 1, f);
	for (i = 0; i < 16; i++)
		r += fread(&(pd->count[i]), sizeof(uint64_t), 1, f);
	r += fread(pd->ptable, sizeof(entry_group_t), ptablesize(pd), f);

	fclose(f);

	return r == 17 + ptablesize(pd);
}

static bool
write_ptable_file(PruneData *pd)
{
	init_env();

	FILE *f;
	char fname[strlen(tabledir)+100];
	int i;
	uint64_t w;

	strcpy(fname, tabledir);
	strcat(fname, "/");
	strcat(fname, pd->filename);

	if ((f = fopen(fname, "wb")) == NULL)
		return false;

	w = fwrite(&(pd->base), sizeof(int), 1, f);
	for (i = 0; i < 16; i++)
		w += fwrite(&(pd->count[i]), sizeof(uint64_t), 1, f);
	w += fwrite(pd->ptable, sizeof(entry_group_t), ptablesize(pd), f);
	fclose(f);

	return w == 17 + ptablesize(pd);
}

