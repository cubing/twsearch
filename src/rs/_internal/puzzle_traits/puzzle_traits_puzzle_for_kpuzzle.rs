use std::hash::BuildHasher;

use cubing::{
    alg::Move,
    kpuzzle::{InvalidAlgError, KPattern, KPuzzle, KTransformation, KTransformationBuffer},
};

use super::{GroupActionPuzzle, HashablePatternPuzzle, MoveCount, SemiGroupActionPuzzle};

impl SemiGroupActionPuzzle for KPuzzle {
    type Pattern = KPattern;
    type Transformation = KTransformation;

    // fn puzzle_default_pattern(&self) -> Self::Pattern {
    //     self.default_pattern()
    // }

    fn move_order(&self, r#move: &Move) -> Result<MoveCount, InvalidAlgError> {
        let transformation = self.puzzle_transformation_from_move(r#move)?;
        let identity_transformation = transformation.kpuzzle().identity_transformation();
        let mut order: i32 = 1;
        let mut current_transformation = KTransformationBuffer::from(transformation.clone());
        while *current_transformation.current() != identity_transformation {
            current_transformation.apply_transformation(&transformation);
            order += 1;
        }
        Ok(order)
    }

    fn do_moves_commute(
        &self,
        move1_info: &crate::_internal::MoveTransformationInfo<Self>,
        move2_info: &crate::_internal::MoveTransformationInfo<Self>,
    ) -> bool {
        move1_info
            .transformation
            .apply_transformation(&move2_info.transformation)
            == move2_info
                .transformation
                .apply_transformation(&move1_info.transformation)
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        self.transformation_from_move(r#move)
    }

    fn pattern_apply_transformation(
        &self, // TODO
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        Some(pattern.apply_transformation(transformation_to_apply)) // TODO
    }

    fn pattern_apply_transformation_into(
        &self, // TODO
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        pattern.apply_transformation_into(transformation_to_apply, into_pattern);
        true
    }
}

impl HashablePatternPuzzle for KPuzzle {
    fn pattern_hash_u64(&self, pattern: &Self::Pattern) -> u64 {
        let h = cityhasher::CityHasher::new();
        h.hash_one(unsafe { pattern.byte_slice() })
    }
}

impl GroupActionPuzzle for KPuzzle {
    // /* TGR: can't do algs in symcoords without additional scaffolding */
    // fn puzzle_transformation_from_alg(
    //     &self,
    //     alg: &Alg,
    // ) -> Result<Self::Transformation, InvalidAlgError> {
    //     self.transformation_from_alg(alg)
    // }

    // /* TGR: can't expose generic transformation, only moves */
    // fn puzzle_identity_transformation(&self) -> Self::Transformation {
    //     self.identity_transformation()
    // }

    /* TGR: okay, but we need to talk about how this relates to search generators */
    /// This includes directly defined moves as well as derived moves.
    fn puzzle_definition_all_moves(&self) -> Vec<&Move> {
        let def = self.definition();
        let moves = def.moves.keys();
        if let Some(derived_moves) = &def.derived_moves {
            moves.chain(derived_moves.keys()).collect()
        } else {
            moves.collect()
        }
    }

    // /* TGR:  no invert in symcoords (and no transformation */
    // fn transformation_invert(transformation: &Self::Transformation) -> Self::Transformation {
    //     transformation.invert()
    // }

    // /* TGR: we can go Pattern x Move -> Pattern and that's it */
    // fn transformation_apply_transformation(
    //     transformation: &Self::Transformation,
    //     transformation_to_apply: &Self::Transformation,
    // ) -> Self::Transformation {
    //     transformation.apply_transformation(transformation_to_apply)
    // }

    // /* TGR: no transformations (but we can "into" a pattern)  */
    // fn transformation_apply_transformation_into(
    //     transformation: &Self::Transformation,
    //     transformation_to_apply: &Self::Transformation,
    //     into_transformation: &mut Self::Transformation,
    // ) {
    //     transformation.apply_transformation_into(transformation_to_apply, into_transformation);
    // }

    // /* TGR:  symcoords are their own hash and don't really hook up to other tables */
    // fn transformation_hash_u64(&self, transformation: &Self::Transformation) -> u64 {
    //     let h = cityhasher::CityHasher::new();
    //     h.hash_one(unsafe { transformation.byte_slice() })
    // }
}
