use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::{BitAnd, Index, IndexMut},
    slice::Iter,
};

use cubing::alg::QuantumMove;

use crate::{
    _internal::{
        puzzle_traits::puzzle_traits::SemiGroupActionPuzzle, search::indexed_vec::IndexedVec,
    },
    whole_number_newtype,
};

use super::search_generators::{MoveTransformationInfo, SearchGenerators};

whole_number_newtype!(MoveClassIndex, usize);

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, Hash, Clone)]
struct MoveClassMask(IndexedVec<MoveClassIndex, bool>);

impl From<Vec<bool>> for MoveClassMask {
    fn from(value: Vec<bool>) -> Self {
        Self(IndexedVec::new(value))
    }
}

impl PartialEq for MoveClassMask {
    fn eq(&self, other: &Self) -> bool {
        // We explicitly avoid implementing `Eq` for `IndexedVec` to avoid accidental unperformant operations, so we have to call into the lowest-level `Vec` here.
        self.0 .0 == other.0 .0
    }
}

impl Eq for MoveClassMask {}

impl BitAnd for &MoveClassMask {
    type Output = MoveClassMask;

    fn bitand(self, rhs: Self) -> MoveClassMask {
        debug_assert_eq!(self.len(), rhs.len());
        self.iter()
            .zip(rhs.iter())
            .map(|(lhs, rhs)| *lhs & *rhs)
            .collect::<Vec<bool>>()
            .into()
    }
}

impl Index<MoveClassIndex> for MoveClassMask {
    type Output = bool;

    fn index(&self, index: MoveClassIndex) -> &bool {
        &self.0[index]
    }
}

impl IndexMut<MoveClassIndex> for MoveClassMask {
    fn index_mut(&mut self, index: MoveClassIndex) -> &mut bool {
        &mut self.0[index]
    }
}

impl MoveClassMask {
    fn iter(&self) -> Iter<bool> {
        self.0 .0.iter()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

whole_number_newtype!(CanonicalFSMState, usize);

pub(crate) const CANONICAL_FSM_START_STATE: CanonicalFSMState = CanonicalFSMState(0);
pub(crate) const ILLEGAL_FSM_STATE: CanonicalFSMState = CanonicalFSMState(0xFFFFFFFF);

#[derive(Default, Debug)]
struct MaskToState(HashMap<MoveClassMask, CanonicalFSMState>);

impl MaskToState {
    pub fn insert(&mut self, mask: MoveClassMask, state: CanonicalFSMState) {
        self.0.insert(mask, state);
    }

    pub fn get(&self, mask: &MoveClassMask) -> Option<CanonicalFSMState> {
        self.0.get(mask).copied() // TODO: figure out how to do this safely and most performantly
    }
}

#[derive(Debug)]
pub struct CanonicalFSM<TPuzzle: SemiGroupActionPuzzle> {
    pub(crate) next_state_lookup:
        IndexedVec<CanonicalFSMState, IndexedVec<MoveClassIndex, CanonicalFSMState>>,

    phantom_data: PhantomData<TPuzzle>,
}

#[derive(Debug, Default)]
pub struct CanonicalFSMConstructionOptions {
    pub forbid_transitions_by_quantums_either_direction: HashSet<(QuantumMove, QuantumMove)>,
}

impl CanonicalFSMConstructionOptions {
    fn is_transition_forbidden<TPuzzle: SemiGroupActionPuzzle>(
        &self,
        move1_info: &MoveTransformationInfo<TPuzzle>,
        move2_info: &MoveTransformationInfo<TPuzzle>,
    ) -> bool {
        self.forbid_transitions_by_quantums_either_direction
            .contains(&(
                move1_info.r#move.quantum.as_ref().clone(),
                move2_info.r#move.quantum.as_ref().clone(),
            ))
            || self
                .forbid_transitions_by_quantums_either_direction
                .contains(&(
                    move2_info.r#move.quantum.as_ref().clone(),
                    move1_info.r#move.quantum.as_ref().clone(),
                ))
    }
}

impl<TPuzzle: SemiGroupActionPuzzle> CanonicalFSM<TPuzzle> {
    /// Pass `Default::default()` as for `options` when no options are needed.
    pub fn new(
        tpuzzle: TPuzzle,
        generators: SearchGenerators<TPuzzle>, // TODO: make this a field in the options?
        options: CanonicalFSMConstructionOptions,
    ) -> CanonicalFSM<TPuzzle> {
        let num_move_classes = generators.by_move_class.len();

        let mut commutes: IndexedVec<MoveClassIndex, MoveClassMask> =
            IndexedVec::new(vec![vec![true; num_move_classes].into(); num_move_classes]);
        let mut forbidden_transitions: IndexedVec<MoveClassIndex, MoveClassMask> =
            IndexedVec::new(vec![vec![false; num_move_classes].into(); num_move_classes]);

        // Written this way so if we later iterate over all moves instead of
        // all move classes. This is because multiples can commute differently than their quantum values.
        // For example:
        // - The standard T-Perm (`R U R' U' R' F R2 U' R' U' R U R' F'`) has order 2.
        // - `R2 U2` has order 6.
        // - T-perm and `(R2 U2)3` commute.
        for i in 0..num_move_classes {
            let i = MoveClassIndex(i);
            for j in 0..num_move_classes {
                let j = MoveClassIndex(j);
                let move1_info = &generators.by_move_class[i][0];
                let move2_info = &generators.by_move_class[j][0];
                if !tpuzzle.do_moves_commute(move1_info, move2_info) {
                    commutes[i][j] = false;
                    commutes[j][i] = false;
                }
                if options.is_transition_forbidden(move1_info, move2_info) {
                    forbidden_transitions[i][j] = true;
                    forbidden_transitions[j][i] = true;
                }
            }
        }

        let mut next_state_lookup: IndexedVec<
            CanonicalFSMState,
            IndexedVec<MoveClassIndex, CanonicalFSMState>,
        > = IndexedVec::default();

        let mut mask_to_state = MaskToState::default();
        mask_to_state.insert(
            vec![false; num_move_classes].into(),
            CANONICAL_FSM_START_STATE,
        );
        // state_to_mask, indexed by state ordinal,  holds the set of move classes in the
        // move sequence so far for which there has not been a subsequent move that does not
        // commute with that move.
        let mut state_to_mask = IndexedVec::<CanonicalFSMState, MoveClassMask>::new(vec![
            vec![false; num_move_classes].into(),
        ]);

        let mut queue_index: CanonicalFSMState = CANONICAL_FSM_START_STATE;
        while usize::from(queue_index) < state_to_mask.len() {
            let mut next_state: IndexedVec<MoveClassIndex, CanonicalFSMState> =
                IndexedVec::new(vec![ILLEGAL_FSM_STATE; num_move_classes]);

            let dequeue_move_class_mask = state_to_mask[queue_index].clone();

            queue_index += CanonicalFSMState(1);

            'outer: for move_class_index in generators.by_move_class.index_iter() {
                for i in 0..num_move_classes {
                    let i = MoveClassIndex(i);
                    // If the transition is forbidden by the options, we can't
                    // move `move_class`
                    if dequeue_move_class_mask[i] && forbidden_transitions[move_class_index][i] {
                        continue 'outer;
                    }

                    // If there's a greater move (multiple) in the state that
                    // commutes with this move's `move_class`, we can't move
                    // `move_class`.
                    if i > move_class_index
                        && dequeue_move_class_mask[i]
                        && commutes[move_class_index][i]
                    {
                        continue 'outer;
                    }
                }
                if dequeue_move_class_mask[move_class_index] {
                    continue;
                }
                let mut next_state_mask = &dequeue_move_class_mask & &commutes[move_class_index];
                next_state_mask[move_class_index] = true;

                // If a pair of bits are set with the same commutating moves, we
                // can clear out the higher ones. This optimization keeps the
                // state count from going exponential for very big cubes.
                for i in 0..num_move_classes {
                    let i = MoveClassIndex(i);
                    if next_state_mask[i] {
                        for j in (i.0 + 1)..num_move_classes {
                            let j = MoveClassIndex(j);
                            if next_state_mask[j] && commutes[i] == commutes[j] {
                                next_state_mask[i] = false;
                            }
                        }
                    }
                }

                let next_move_mask_class = next_state_mask;

                next_state[move_class_index] = match mask_to_state.get(&next_move_mask_class) {
                    None => {
                        let next_state = CanonicalFSMState(state_to_mask.0.len());
                        mask_to_state.insert(next_move_mask_class.clone(), next_state);
                        state_to_mask.push(next_move_mask_class);
                        next_state
                    }
                    Some(state) => state,
                };
            }
            next_state_lookup.push(next_state);
        }

        Self {
            next_state_lookup,
            phantom_data: PhantomData,
        }
    }

    pub(crate) fn next_state(
        &self,
        current_fsm_state: CanonicalFSMState,
        move_class_index: MoveClassIndex,
    ) -> Option<CanonicalFSMState> {
        match self.next_state_lookup[current_fsm_state][move_class_index] {
            ILLEGAL_FSM_STATE => None,
            state => Some(state),
        }
    }
}
