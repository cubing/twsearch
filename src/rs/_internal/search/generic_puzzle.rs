use std::fmt::Debug;

use cubing::{
    alg::{Alg, Move},
    kpuzzle::InvalidAlgError,
};

// TODO: split this into 3 related traits.
pub trait GenericPuzzleCore: Debug {
    type Pattern: Eq + Clone + Debug;
    /// This is a proper "transformation" (such as a permutation) in the general
    /// case, but for `GenericPuzzleCore` it can be anything that is applied to a
    /// pattern, such as:
    ///
    /// - A [`Move`]
    /// - An index or reference into an array that encodes how to apply it
    type Transformation: Eq + Clone + Debug;

    /********* Functions "defined on the puzzle". ********/

    fn puzzle_default_pattern(&self) -> Self::Pattern;

    /********* Functions "defined on the transformation". ********/

    fn puzzle_transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<Self::Transformation, InvalidAlgError>;

    /********* Functions "defined on the pattern". ********/

    fn pattern_apply_transformation(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Pattern;
    fn pattern_apply_transformation_into(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    );

    // fn pattern_puzzle(pattern: &Self::Pattern) -> &Self; // TODO: add an additional trait for this.
    fn pattern_hash_u64(pattern: &Self::Pattern) -> u64;
}

pub trait GenericPuzzle: GenericPuzzleCore + Clone {
    /********* Functions "defined on the puzzle". ********/

    fn puzzle_transformation_from_alg(
        &self,
        alg: &Alg,
    ) -> Result<Self::Transformation, InvalidAlgError>;
    fn puzzle_identity_transformation(&self) -> Self::Transformation; // TODO: also define this on `KPuzzle` itself.
    fn puzzle_definition_moves(&self) -> Vec<&Move>;

    /********* Functions "defined on the transformation". ********/

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
