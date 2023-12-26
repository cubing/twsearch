#ifndef TEST_H
#include "generatingset.h"
#include "puzdef.h"
/*
 *   A couple of test routines.
 */
void timingtest(puzdef &pd);
void solvetest(puzdef &pd, generatingset *gs = 0);
void ensure_test_is_linked();
#define TEST_H
#endif
