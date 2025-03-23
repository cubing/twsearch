use std::marker::PhantomData;

use num_integer::lcm;

use crate::_internal::{
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::{
        coordinates::pattern_deriver::{DerivedPatternPuzzle, PatternDeriver},
        move_count::MoveCount,
    },
};

use super::CompoundPuzzle;

// // Trait to group a puzzle with a derived pattern to make dependent type signatures shorter.
// pub trait PuzzleWithDerivedPattern: Clone + Debug {
//     type DerivationSourcePuzzle: SemiGroupActionPuzzle;
//     type DerivedPuzzle: SemiGroupActionPuzzle;

//     fn derive_pattern(pattern: Derivation)
// }

#[derive(Clone, Debug)]
pub struct CompoundDerivedPuzzle<
    TPuzzle: SemiGroupActionPuzzle,
    TDerivedPatternPuzzle0: DerivedPatternPuzzle<TPuzzle>,
    TDerivedPatternPuzzle1: DerivedPatternPuzzle<TPuzzle>,
> {
    pub tpuzzle: CompoundPuzzle<TDerivedPatternPuzzle0, TDerivedPatternPuzzle1>,
    // pub search_generators_t1: SearchGenerators<T1::DerivationSourcePuzzle>,
    // pub search_generators_t2: SearchGenerators<T2::DerivationSourcePuzzle>,
    pub phantom_data: PhantomData<TPuzzle>,
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPatternPuzzle0: DerivedPatternPuzzle<TPuzzle>,
        TDerivedPatternPuzzle1: DerivedPatternPuzzle<TPuzzle>,
    > PatternDeriver<TPuzzle>
    for CompoundDerivedPuzzle<TPuzzle, TDerivedPatternPuzzle0, TDerivedPatternPuzzle1>
{
    type DerivedPattern = <CompoundPuzzle<
        TDerivedPatternPuzzle0,
        TDerivedPatternPuzzle1,
    > as SemiGroupActionPuzzle>::Pattern;

    fn derive_pattern(
        &self,
        source_puzzle_pattern: &<TPuzzle as SemiGroupActionPuzzle>::Pattern,
    ) -> Option<Self::DerivedPattern> {
        let pattern0: TDerivedPatternPuzzle0::Pattern = self
            .tpuzzle
            .tpuzzle0
            .derive_pattern(source_puzzle_pattern)?;
        let pattern1: TDerivedPatternPuzzle1::Pattern = self
            .tpuzzle
            .tpuzzle1
            .derive_pattern(source_puzzle_pattern)?;
        Some((pattern0, pattern1))
    }
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPatternPuzzle0: DerivedPatternPuzzle<TPuzzle>,
        TDerivedPatternPuzzle1: DerivedPatternPuzzle<TPuzzle>,
    > DerivedPatternPuzzle<TPuzzle>
    for CompoundDerivedPuzzle<TPuzzle, TDerivedPatternPuzzle0, TDerivedPatternPuzzle1>
{
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPatternPuzzle0: DerivedPatternPuzzle<TPuzzle>,
        TDerivedPatternPuzzle1: DerivedPatternPuzzle<TPuzzle>,
    > SemiGroupActionPuzzle
    for CompoundDerivedPuzzle<TPuzzle, TDerivedPatternPuzzle0, TDerivedPatternPuzzle1>
{
    type Pattern = <CompoundPuzzle<
    TDerivedPatternPuzzle0,
    TDerivedPatternPuzzle1,
> as SemiGroupActionPuzzle>::Pattern;
    type Transformation = <CompoundPuzzle<
    TDerivedPatternPuzzle0,
    TDerivedPatternPuzzle1,
> as SemiGroupActionPuzzle>::Transformation;

    fn move_order(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<crate::_internal::search::move_count::MoveCount, cubing::kpuzzle::InvalidAlgError>
    {
        Ok(MoveCount(lcm(
            self.tpuzzle.tpuzzle0.move_order(r#move)?.0,
            self.tpuzzle.tpuzzle1.move_order(r#move)?.0,
        )))
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
        Ok((
            self.tpuzzle
                .tpuzzle0
                .puzzle_transformation_from_move(r#move)?,
            self.tpuzzle
                .tpuzzle1
                .puzzle_transformation_from_move(r#move)?,
        ))
    }

    fn do_moves_commute(
        &self,
        move1_info: &crate::_internal::canonical_fsm::search_generators::MoveTransformationInfo<
            Self,
        >,
        move2_info: &crate::_internal::canonical_fsm::search_generators::MoveTransformationInfo<
            Self,
        >,
    ) -> bool {
        self.tpuzzle.tpuzzle0.do_moves_commute(
            &self.tpuzzle.search_generators_t0.flat[move1_info.flat_move_index],
            &self.tpuzzle.search_generators_t0.flat[move2_info.flat_move_index],
        ) && self.tpuzzle.tpuzzle1.do_moves_commute(
            &self.tpuzzle.search_generators_t1.flat[move1_info.flat_move_index],
            &self.tpuzzle.search_generators_t1.flat[move2_info.flat_move_index],
        )
    }

    fn pattern_apply_transformation(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        let pattern0 = self
            .tpuzzle
            .tpuzzle0
            .pattern_apply_transformation(&pattern.0, &transformation_to_apply.0)?;
        let pattern1 = self
            .tpuzzle
            .tpuzzle1
            .pattern_apply_transformation(&pattern.1, &transformation_to_apply.1)?;
        Some((pattern0, pattern1))
    }

    fn pattern_apply_transformation_into(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        self.tpuzzle.tpuzzle0.pattern_apply_transformation_into(
            &pattern.0,
            &transformation_to_apply.0,
            &mut into_pattern.0,
        ) && self.tpuzzle.tpuzzle1.pattern_apply_transformation_into(
            &pattern.1,
            &transformation_to_apply.1,
            &mut into_pattern.1,
        )
    }
}

// // TODO
// unsafe impl<T1: PuzzleWithDerivedPattern, T2: PuzzleWithDerivedPattern> Sync
//     for CompoundDerivedPuzzle<T1, T2>
// {
// }

// // TODO
// unsafe impl<T1: PuzzleWithDerivedPattern, T2: PuzzleWithDerivedPattern> Send
//     for CompoundDerivedPuzzle<T1, T2>
// {
// }

// impl<T1: PuzzleWithDerivedPattern, T2: PuzzleWithDerivedPattern> SemiGroupActionPuzzle
//     for CompoundDerivedPuzzle<T1, T2>
// {
//     type Pattern = (
//         <T1 as PuzzleWithDerivedPattern>::DerivedPattern,
//         <T2 as PuzzleWithDerivedPattern>::DerivedPattern,
//     );
//     type Transformation = (
//         <<T1 as PuzzleWithDerivedPattern>::DerivationSourcePuzzle as SemiGroupActionPuzzle>::Transformation,
//         <<T2 as PuzzleWithDerivedPattern>::DerivationSourcePuzzle as SemiGroupActionPuzzle>::Transformation,
//     );

//     fn move_order(
//         &self,
//         r#move: &cubing::alg::Move,
//     ) -> Result<crate::_internal::search::move_count::MoveCount, cubing::kpuzzle::InvalidAlgError>
//     {
//     }

//     fn puzzle_transformation_from_move(
//         &self,
//         r#move: &cubing::alg::Move,
//     ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
//     }

//     fn do_moves_commute(
//         &self,
//         move1_info: &MoveTransformationInfo<Self>,
//         move2_info: &MoveTransformationInfo<Self>,
//     ) -> bool {
//     }

//     fn pattern_apply_transformation(
//         // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
//         // Then again, this might turn out to be necessary for similar high-performance implementations.
//         &self,
//         pattern: &Self::Pattern,
//         transformation_to_apply: &Self::Transformation,
//     ) -> Option<Self::Pattern> {
//         let pattern1 = self
//             .tpuzzle0
//             .pattern_apply_transformation(&pattern.0, &transformation_to_apply.0)?;
//         let pattern2 = self
//             .tpuzzle1
//             .pattern_apply_transformation(&pattern.1, &transformation_to_apply.1)?;
//         Some((pattern1, pattern2))
//     }

//     fn pattern_apply_transformation_into(
//         // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
//         // Then again, this might turn out to be necessary for similar high-performance implementations.
//         &self,
//         pattern: &Self::Pattern,
//         transformation_to_apply: &Self::Transformation,
//         into_pattern: &mut Self::Pattern,
//     ) -> bool {
//         self.tpuzzle0.pattern_apply_transformation_into(
//             &pattern.0,
//             &transformation_to_apply.0,
//             &mut into_pattern.0,
//         ) && self.tpuzzle1.pattern_apply_transformation_into(
//             &pattern.1,
//             &transformation_to_apply.1,
//             &mut into_pattern.1,
//         )
//     }
// }

// // pub trait DefaultSearchAdaptations<TPuzzle: SemiGroupActionPuzzle> {
// //     type Adaptations: SearchAdaptations<TPuzzle>;
// // }

// // impl<T1: PuzzleWithDerivedPattern, T2: PuzzleWithDerivedPattern>
// //     DefaultSearchAdaptations<CompoundDerivedPuzzle<T1, T2>> for CompoundDerivedPuzzle<T1, T2>
// // {
// //     type Adaptations = CompoundDerivedPuzzleDefaultSearchAdaptations<CompoundDerivedPuzzle<T1, T2>>;
// // }

// // Convenience type
// pub type TripleCompoundPuzzle<
//     T1: PuzzleWithDerivedPattern,
//     T2: PuzzleWithDerivedPattern,
//     T3: PuzzleWithDerivedPattern,
// > = CompoundDerivedPuzzle<T1, CompoundDerivedPuzzle<T2, T3>>;

// // Convenience type
// pub type QuadrupleCompoundPuzzle<
//     T1: PuzzleWithDerivedPattern,
//     T2: PuzzleWithDerivedPattern,
//     T3: PuzzleWithDerivedPattern,
//     T4: PuzzleWithDerivedPattern,
// > = CompoundDerivedPuzzle<CompoundDerivedPuzzle<T1, T2>, CompoundDerivedPuzzle<T3, T4>>;

// struct CompoundDerivedPuzzleSearchWithHashPruneTable<
//     T1: PuzzleWithDerivedPattern,
//     T2: PuzzleWithDerivedPattern,
// > {
//     pub tpuzzle: CompoundDerivedPuzzle<T1, T2>,

//     pub phase_name: String,
//     pub iterative_deepening_search: IterativeDeepeningSearch<CompoundDerivedPuzzle<T1, T2>>,
//     // TODO: support passing these in dynamically somehow
//     pub individual_search_options: IndividualSearchOptions,
// }

// #[derive(Default)]
// pub struct CompoundDerivedPuzzleConstructionOptions {
//     pub search_logger: Option<SearchLogger>,
//     pub individual_search_options: Option<IndividualSearchOptions>,
//     pub iterative_deepening_search_construction_options:
//         Option<IterativeDeepeningSearchConstructionOptions>,
// }

// impl<T1: PuzzleWithDerivedPattern + 'static, T2: PuzzleWithDerivedPattern + 'static>
//     CompoundDerivedPuzzleSearchWithHashPruneTable<T1, T2>
// where
//     T1::DerivationSourcePuzzle: HashablePatternPuzzle,
//     T2::DerivationSourcePuzzle: HashablePatternPuzzle,
// {
//     /// The caller must ensure that the ` are safe to transfer to other puzzles.
//     fn try_new(
//         tpuzzle: CompoundDerivedPuzzle<T1, T2>,
//         phase_name: String,
//         generator_moves: Vec<Move>,
//         options: CompoundDerivedPuzzleConstructionOptions,
//         target_patterns: Vec<<CompoundDerivedPuzzle<T1, T2> as SemiGroupActionPuzzle>::Pattern>,
//     ) -> Result<Self, SearchError> {
//         let iterative_deepening_search =
//             IterativeDeepeningSearch::<CompoundDerivedPuzzle<T1, T2>>::try_new_prune_table_construction_shim::<HashPruneTable<CompoundDerivedPuzzle<T1, T2>>>(
//                 tpuzzle.clone(),
//                 generator_moves,
//                 target_patterns,
//                 options
//                     .iterative_deepening_search_construction_options
//                     .unwrap_or_default(),
//                     None,
//             )?;

//         Ok(Self {
//             iterative_deepening_search,
//             tpuzzle,
//             phase_name,
//             individual_search_options: options.individual_search_options.unwrap_or_default(),
//         })
//     }
// }

// impl<T1: PuzzleWithDerivedPattern, T2: PuzzleWithDerivedPattern> HashablePatternPuzzle
//     for CompoundDerivedPuzzle<T1, T2>
// where
//     T1::DerivationSourcePuzzle: HashablePatternPuzzle,
//     T2::DerivationSourcePuzzle: HashablePatternPuzzle,
// {
//     fn pattern_hash_u64(&self, pattern: &Self::Pattern) -> u64 {
//         // TODO: make this more effficient for stacked compound puzzles (e.g. triple or quadruple)
//         let mut h = cityhasher::CityHasher::new();
//         h.write(&self.tpuzzle0.pattern_hash_u64(&pattern.0).to_le_bytes());
//         h.write(&self.tpuzzle1.pattern_hash_u64(&pattern.1).to_le_bytes());
//         h.finish()
//     }
// }

// // TODO
// unsafe impl<T1: PuzzleWithDerivedPattern, T2: PuzzleWithDerivedPattern> Sync
//     for CompoundDerivedPuzzleSearchWithHashPruneTable<T1, T2>
// {
// }

// // TODO
// unsafe impl<T1: PuzzleWithDerivedPattern, T2: PuzzleWithDerivedPattern> Send
//     for CompoundDerivedPuzzleSearchWithHashPruneTable<T1, T2>
// {
// }

// impl<T1: PuzzleWithDerivedPattern, T2: PuzzleWithDerivedPattern>
//     SearchPhase<CompoundDerivedPuzzle<T1, T2>>
//     for CompoundDerivedPuzzleSearchWithHashPruneTable<T1, T2>
// {
//     fn phase_name(&self) -> &str {
//         &self.phase_name
//     }

//     fn first_solution(
//         &mut self,
//         phase_search_pattern: &<CompoundDerivedPuzzle<T1, T2> as SemiGroupActionPuzzle>::Pattern,
//     ) -> Result<Option<cubing::alg::Alg>, crate::_internal::errors::SearchError> {
//         // TODO: can we avoid a clone of `individual_search_options`?
//         Ok(self
//             .iterative_deepening_search
//             .search(
//                 &phase_search_pattern,
//                 self.individual_search_options.clone(),
//             )
//             .next())
//     }
// }
