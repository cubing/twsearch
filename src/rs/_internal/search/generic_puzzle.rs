use std::fmt::Debug;

use cubing::{
    alg::{Alg, Move},
    kpuzzle::{InvalidAlgError, KPattern, KPuzzle, KTransformation},
};

// TODO: split this into 3 related traits.
pub trait GenericPuzzle: Clone + Debug {
    type Pattern: Eq + Clone + Debug;
    type Transformation: Eq + Clone + Debug;

    // Functions "defined on the puzzle".
    fn puzzle_default_pattern(&self) -> Self::Pattern;
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

    // Functions "defined on the pattern".
    fn transformation_puzzle(transformation: &Self::Transformation) -> &Self;
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

    // Functions "defined on the transformation".
    fn pattern_puzzle(pattern: &Self::Pattern) -> &Self;
    fn pattern_apply_transformation(
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Pattern;
    fn pattern_apply_transformation_into(
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    );
    fn pattern_hash_u64(pattern: &Self::Pattern) -> u64;
}

impl GenericPuzzle for KPuzzle {
    type Pattern = KPattern;
    type Transformation = KTransformation;

    fn puzzle_default_pattern(&self) -> Self::Pattern {
        self.default_pattern()
    }

    fn puzzle_transformation_from_alg(
        &self,
        alg: &Alg,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        self.transformation_from_alg(alg)
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        self.transformation_from_move(r#move)
    }

    fn puzzle_identity_transformation(&self) -> Self::Transformation {
        self.identity_transformation()
    }

    fn puzzle_definition_moves(&self) -> Vec<&Move> {
        let def = self.definition();
        let moves = def.moves.keys();
        if let Some(derived_moves) = &def.derived_moves {
            moves.chain(derived_moves.keys()).collect()
        } else {
            moves.collect()
        }
    }

    fn transformation_puzzle(transformation: &Self::Transformation) -> &Self {
        transformation.kpuzzle()
    }

    fn transformation_invert(transformation: &Self::Transformation) -> Self::Transformation {
        transformation.invert()
    }

    fn transformation_apply_transformation(
        transformation: &Self::Transformation,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Transformation {
        transformation.apply_transformation(transformation_to_apply)
    }

    fn transformation_apply_transformation_into(
        transformation: &Self::Transformation,
        transformation_to_apply: &Self::Transformation,
        into_transformation: &mut Self::Transformation,
    ) {
        transformation.apply_transformation_into(transformation_to_apply, into_transformation);
    }

    fn transformation_hash_u64(transformation: &Self::Transformation) -> u64 {
        transformation.hash()
    }

    fn pattern_puzzle(pattern: &Self::Pattern) -> &Self {
        pattern.kpuzzle()
    }

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

    fn pattern_hash_u64(pattern: &Self::Pattern) -> u64 {
        pattern.hash()
    }
}
