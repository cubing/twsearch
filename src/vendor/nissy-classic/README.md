# Nissy

A Rubik's cube solver and FMC assistant.
For optimal HTM solving Nissy uses techniques from Herbert Kociemba's
[Cube Explorer](http://kociemba.org/cube.htm) and Tomas Rokicki's
[nxopt](https://github.com/rokicki/cube20src/blob/master/nxopt.md).
With 4 cores at 2.5GHz and using about 3Gb of RAM, Nissy can find an
optimal solution in about a minute on average.

Nissy can also solve many different substeps of Thistlethwaite's algorithm
(DR/HTR), and can use NISS (Normal-Inverse Scramble Switch).
It can be useful to analyze your DR solves (and more, once I implement more features).

You can get Nissy from [nissy.tronto.net](https://nissy.tronto.net).
The download links and installation instructions can be found on the
[download page](https://nissy.tronto.net/download).

**Note:** This branch of nissy will get only bugfixes and small additions.
For more info, see [nissy](https://git.tronto.net/nissy).

## Structure of the code

You can find all the source code in the `src` folder.
I strived to keep it legible but I did not write many comments (barely any at all).
I'll try to explain here the main parts of the program.

### Cube, moves and transformations

There are many ways to represent a cube. In Nissy I use two:

* An array representation `CubeArray`: 3 arrays representing the permutation
of corners, edges and centers and 2 arrays for the orientation of corners and edges.
* An 11-integers representation `Cube`: 3 integers for edge orientation (with respect
to the three axes), 3 for corner orientation, and so on. Edge permutation is a bit
complicated because encoding 12 factorial as a single number is too large for some
practical reasons, so I use 3 integers for that.

Moves are easy to apply on the array form, but they are slow. So `moves.c`
contains the instructions to create all the transition tables necessary
to get the next position for the cube with just 11 lookup operations
(one for each of the 11 integers in the second representation).
These transition tables are saved in the `mtables` file in the
`tables` folder in binary format.

The 11 integers are obviously redundant, but keeping all of them makes it easy
to apply transformations. A transformation is a rotation of the whole cube, possibly
combined with a mirror operation. Applying a transformation to a cube (say obtained
by applying a scramble to the solved cube) means applying the transformation to a
solved cube, then the scramble and then the inverse of the transformation
(i.e. conjugating by it).

### Coordinates and pruning tables

A *coordinate* consists of a function that takes a cube (in the 11-integer
representation) and return an (unsigned, 64-bit) integer. They are used
to "linearize" a cube and build pruning tables, which speed up significantly the
solving process. To be able to access the pruning table quickly, the function
needs to be very fast (e.g. it should not convert between the two representations
of the cube if not necessary).

Some coordinates make use of symmetries to reduce the size of the resulting
pruning table. Unfortunately this complicates the code a lot, but it is a huge
advantage: it reduces by a factor of about 16 the pruning table size.

Pruning tables are related to a specific step, a moveset and a coordinate. They
contain one value from 0 to 15 (4 bits) for each possible value for the coordinate,
which is less or equal than the minimum number of moves required to solve the
given step with the given moveset for a cube which has the given coordinate. For example,
say the coordinate `neo` gives the number of non-oriented edges (say with respect to
F/B). Then the possible values for the coordinate are 0,2,4,...,12. An associated
pruning table to solving EO with HTM moveset and this coordinate would have values 0
(for `neo=0`), 3 (for `neo=2`), 1 (for `neo=4`)...

The values for most pruning tables are memorized modulo 16, so they only occupy
4 bits per entry, and values larger than 15 are saved as 15. This is good enough
for most applications.
Some large tables are memorized in compact form using only 2 bits, similarly
to what [nxopt](https://github.com/rokicki/cube20src/blob/master/nxopt.md) does:
a base value `b` is picked and a value of `n` is saved as `MIN(3,MAX(0,n-b))`.
When a value of `v=1,2,3` is read it is simply returned as `v+b`, while if
`0` is a successive lookup to a fallback table is performed. The base value `b`
is picked to maximize the sum frequency of the values `1,2,3`.

In order to generate the pruning tables, it is necessary to be able to move
a transform a coordinate; it is possible to do so without passing through a
complete cube representations, in a way similar to what Cube Explorer does.
This used to be different before version 2.1 (June 2022).

More documentation on this and on the different types of coordinates (base
vs composed) is work in progress.

### Solving

Solving is implemented as a generic function that takes both a step and
a (scrambled) cube as input, as well as some extra parameters that say e.g.
how many solution one wants. A step consists, among other things, of
an estimator function that, given a cube, gives a lower bound for the number
of moves needed to complete the step. Many of these estimators simply
look up the corresponding values in the appropriate pruning table.

