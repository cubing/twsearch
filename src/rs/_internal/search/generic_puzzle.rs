use std::fmt::Debug;

use cubing::{
    alg::{Alg, Move},
    kpuzzle::InvalidAlgError,
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

    // Functions "defined on the pattern".
    // fn pattern_puzzle(pattern: &Self::Pattern) -> &Self; // TODO: add an additional trait for this.
    fn pattern_hash_u64(pattern: &Self::Pattern) -> u64;
}

pub trait GenericPuzzle: GenericPuzzleCore {
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

    // Functions "defined on the pattern".
    fn pattern_apply_transformation_into(
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    );
}
