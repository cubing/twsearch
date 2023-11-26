use cubing::{
    alg::{Alg, Move},
    kpuzzle::{InvalidAlgError, KPattern, KPuzzle, KTransformation},
};

pub trait GenericPuzzle {
    // Define generic types here which methods will be able to utilize.
    type Pattern: Eq;
    type Transformation;
    // type Transformation;

    fn puzzle_default_pattern(&self) -> Self::Pattern;
    fn puzzle_transformation_from_alg(
        &self,
        alg: &Alg,
    ) -> Result<Self::Transformation, InvalidAlgError>;
    fn puzzle_transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<Self::Transformation, InvalidAlgError>;
    fn puzzle_identity_transformation(&self, r#move: &Move) -> Self::Transformation; // TODO: also define this on `KPuzzle` itself.

    fn transformation_kpuzzle(transformation: &Self::Transformation) -> &Self;
    fn transformation_invert(transformation: &Self::Transformation) -> Self::Transformation;
    fn transformation_apply_transformation(
        transformation: &Self::Transformation,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Transformation;

    fn pattern_kpuzzle(pattern: &Self::Pattern) -> &Self;
    fn pattern_apply_transformation(
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Pattern;

    fn TODO_get_kpuzzle(&self) -> &KPuzzle;
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

    fn puzzle_identity_transformation(&self, r#move: &Move) -> Self::Transformation {
        self.identity_transformation()
    }

    fn transformation_kpuzzle(transformation: &Self::Transformation) -> &Self {
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

    fn pattern_kpuzzle(pattern: &Self::Pattern) -> &Self {
        pattern.kpuzzle()
    }

    fn pattern_apply_transformation(
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Pattern {
        pattern.apply_transformation(transformation_to_apply)
    }

    #[allow(non_snake_case)]
    fn TODO_get_kpuzzle(&self) -> &KPuzzle {
        return self;
    }
}
