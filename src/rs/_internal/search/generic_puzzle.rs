use std::fmt::Debug;

use cubing::{
    alg::{Alg, Move},
    kpuzzle::{InvalidAlgError, KPattern, KPuzzle, KTransformation},
};

// TODO: split this into 3 related traits.
pub trait GenericPuzzleCore: Clone + Debug {
    type Pattern: Eq + Clone + Debug;
    type Transformation: Eq + Clone + Debug;

    // Functions "defined on the transformation".
    // fn pattern_puzzle(pattern: &Self::Pattern) -> &Self; // TODO: add an additional trait for this.
    fn pattern_apply_transformation(
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Pattern;
    fn pattern_apply_transformation_into(
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    );

    // Functions "defined on the pattern".
    // fn pattern_puzzle(pattern: &Self::Pattern) -> &Self; // TODO: add an additional trait for this.
    fn pattern_hash_u64(pattern: &Self::Pattern) -> u64;
}

pub trait GenericPuzzle: Clone + Debug + GenericPuzzleCore {
    // Functions "defined on the puzzle".
    fn puzzle_default_pattern(&self) -> Self::Pattern;

    // Functions "defined on the puzzle".
    fn puzzle_transformation_from_alg(
        &self,
        alg: &Alg,
    ) -> Result<Self::Transformation, InvalidAlgError>;
    fn puzzle_transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<Self::Transformation, InvalidAlgError>;
    fn puzzle_identity_transformation(&self) -> Self::Transformation; // TODO: also define this on `KPuzzle` itself.
    fn puzzle_definition_moves(&self) -> Vec<&Move>;

    // Functions "defined on the transformation".
    // fn transformation_puzzle(transformation: &Self::Transformation) -> &Self; // TODO: add an additional trait for this.
    fn transformation_invert(transformation: &Self::Transformation) -> Self::Transformation;
    fn transformation_apply_transformation(
        transformation: &Self::Transformation,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Transformation;
    fn transformation_apply_transformation_into(
        transformation: &Self::Transformation,
        transformation_to_apply: &Self::Transformation,
        into_transformation: &mut Self::Transformation,
    );
    fn transformation_hash_u64(transformation: &Self::Transformation) -> u64;
    // TODO: efficient `order` function?
}

impl GenericPuzzleCore for KPuzzle {
    type Pattern = KPattern;
    type Transformation = KTransformation;

    // Functions "defined on the pattern".
    fn pattern_apply_transformation(
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Pattern {
        pattern.apply_transformation(transformation_to_apply)
    }
    fn pattern_apply_transformation_into(
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) {
        pattern.apply_transformation_into(transformation_to_apply, into_pattern);
    }

    /* TGR: can't hash patterns (we can but we don't want to) */
    fn pattern_hash_u64(pattern: &Self::Pattern) -> u64 {
        pattern.hash()
    }
}

impl GenericPuzzle for KPuzzle {
    fn puzzle_default_pattern(&self) -> Self::Pattern {
        self.default_pattern()
    }

    /* TGR: can't do algs in symcoords without additional scaffolding */
    fn puzzle_transformation_from_alg(
        &self,
        alg: &Alg,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        self.transformation_from_alg(alg)
    }

    /* TGR: can't expose generic transformation, only moves */
    fn puzzle_transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        self.transformation_from_move(r#move)
    }

    /* TGR: can't expose generic transformation, only moves */
    fn puzzle_identity_transformation(&self) -> Self::Transformation {
        self.identity_transformation()
    }

    /* TGR: okay, but we need to talk about how this relates to search generators */
    fn puzzle_definition_moves(&self) -> Vec<&Move> {
        let def = self.definition();
        let moves = def.moves.keys();
        if let Some(derived_moves) = &def.derived_moves {
            moves.chain(derived_moves.keys()).collect()
        } else {
            moves.collect()
        }
    }

    /* TGR:  no invert in symcoords (and no transformation */
    fn transformation_invert(transformation: &Self::Transformation) -> Self::Transformation {
        transformation.invert()
    }

    /* TGR: we can go Pattern x Move -> Pattern and that's it */
    fn transformation_apply_transformation(
        transformation: &Self::Transformation,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Transformation {
        transformation.apply_transformation(transformation_to_apply)
    }

    /* TGR: no transformations (but we can "into" a pattern)  */
    fn transformation_apply_transformation_into(
        transformation: &Self::Transformation,
        transformation_to_apply: &Self::Transformation,
        into_transformation: &mut Self::Transformation,
    ) {
        transformation.apply_transformation_into(transformation_to_apply, into_transformation);
    }

    /* TGR:  symcoords are their own hash and don't really hook up to other tables */
    fn transformation_hash_u64(transformation: &Self::Transformation) -> u64 {
        transformation.hash()
    }
}

/*
impl GenericPuzzle for Phase2SymmetryTables {
    type Pattern = Phase2CoordinateSet;
    type Transformation = usize;
    fn puzzle_default_pattern(&self) -> Self::Pattern {
        Phase2SolvedState
    }

}
*/
