use std::{
    collections::HashMap,
    ops::{AddAssign, BitAndAssign},
};

use cubing::kpuzzle::KTransformation;

use crate::_internal::{SearchError, SearchGenerators};

const MAX_NUM_MOVE_CLASSES: usize = usize::BITS as usize;

#[derive(Clone, Copy, Debug)]
pub struct MoveClassIndex(pub usize);

// Bit N is indexed by a `MoveClass` value of N.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct MoveClassMask(u64);

impl BitAndAssign for MoveClassMask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 = self.0 & rhs.0
    }
}

fn do_transformations_commute(t1: &KTransformation, t2: &KTransformation) -> bool {
    t1.apply_transformation(t2) == t2.apply_transformation(t1)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CanonicalFSMState(pub usize);
pub(crate) const CANONICAL_FSM_START_STATE: CanonicalFSMState = CanonicalFSMState(0);
pub(crate) const ILLEGAL_FSM_STATE: CanonicalFSMState = CanonicalFSMState(0xFFFFFFFF);

impl From<CanonicalFSMState> for usize {
    fn from(value: CanonicalFSMState) -> Self {
        value.0
    }
}

impl AddAssign for CanonicalFSMState {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

#[derive(Default, Debug)]
struct MaskToState(HashMap<MoveClassMask, CanonicalFSMState>);

impl MaskToState {
    pub fn insert(&mut self, mask: MoveClassMask, state: CanonicalFSMState) {
        self.0.insert(mask, state);
    }

    pub fn get(&self, mask: MoveClassMask) -> Option<CanonicalFSMState> {
        self.0.get(&mask).copied() // TODO: figure out how to do this safely and most performantly
    }
}

#[derive(Default, Debug)]
struct StateToMask(Vec<MoveClassMask>);

impl StateToMask {
    pub fn new(initial_value: MoveClassMask) -> StateToMask {
        Self(vec![initial_value])
    }

    // Push the next value (useful while constructing in order.)
    pub fn push(&mut self, mask: MoveClassMask) {
        self.0.push(mask);
    }

    pub fn set(&mut self, state: CanonicalFSMState, value: MoveClassMask) {
        self.0[state.0] = value;
    }

    pub fn get(&self, state: CanonicalFSMState) -> MoveClassMask {
        self.0[state.0]
    }
}

#[derive(Debug)]
pub struct CanonicalFSM {
    // disallowed_move_classes, indexed by state ordinal, holds the set of move classes that should
    // not be made from this state.
    // disallowed_move_classes: StateToMask,
    // Indexed by [CanonicalFSMState][MoveClassIndex]
    pub(crate) next_state_lookup: Vec<Vec<CanonicalFSMState>>,
    // commutes: Vec<MoveClassMask>,
    pub(crate) move_class_indices: Vec<MoveClassIndex>,
}

impl CanonicalFSM {
    // TODO: Return a more specific error.
    pub fn try_new(generators: SearchGenerators) -> Result<CanonicalFSM, SearchError> {
        let num_move_classes = generators.grouped.len();
        if num_move_classes > MAX_NUM_MOVE_CLASSES {
            return Err(SearchError {
                description: "Too many move classes!".to_owned(),
            });
        }
        let move_class_indices: Vec<MoveClassIndex> =
            (0..num_move_classes).map(MoveClassIndex).collect();

        let mut commutes: Vec<MoveClassMask> =
            vec![MoveClassMask((1 << num_move_classes) - 1); num_move_classes];

        // Written this way so if we later iterate over all moves instead of
        // all move classes. This is because multiples can commute differently than their quantum values.
        // For example:
        // - The standard T-Perm (`R U R' U' R' F R2 U' R' U' R U R' F'`) has order 2.
        // - `R2 U2` has order 6.
        // - T-perm and `(R2 U2)3` commute.
        for i in 0..num_move_classes {
            for j in 0..num_move_classes {
                if !do_transformations_commute(
                    &generators.grouped[i][0].transformation,
                    &generators.grouped[j][0].transformation,
                ) {
                    commutes[i] &= MoveClassMask(!(1 << j));
                    commutes[j] &= MoveClassMask(!(1 << i));
                }
            }
        }

        let mut next_state_lookup: Vec<Vec<CanonicalFSMState>> = Vec::new();

        let mut mask_to_state = MaskToState::default();
        mask_to_state.insert(MoveClassMask(0), CANONICAL_FSM_START_STATE);
        let mut state_to_mask = StateToMask::new(MoveClassMask(0));
        // state_to_mask, indexed by state ordinal,  holds the set of move classes in the
        // move sequence so far for which there has not been a subsequent move that does not
        // commute with that move.
        let mut disallowed_move_classes = StateToMask::new(MoveClassMask(0));

        let mut queue_index: CanonicalFSMState = CANONICAL_FSM_START_STATE;
        while Into::<usize>::into(queue_index) < state_to_mask.0.len() {
            let mut next_state: Vec<CanonicalFSMState> = vec![ILLEGAL_FSM_STATE; num_move_classes];

            let dequeue_move_class_mask: MoveClassMask = state_to_mask.get(queue_index);
            disallowed_move_classes.push(MoveClassMask(0));

            queue_index += CanonicalFSMState(1);
            let from_state = queue_index;

            for move_class_index in &move_class_indices {
                // If there's a greater move (multiple) in the state that
                // commutes with this move's `move_class`, we can't move
                // `move_class`.
                if (dequeue_move_class_mask.0 & commutes[move_class_index.0].0)
                    >> (move_class_index.0 + 1)
                    != 0
                {
                    let new_value = MoveClassMask(
                        disallowed_move_classes.get(from_state).0 | (1 << move_class_index.0),
                    );
                    disallowed_move_classes.set(from_state, new_value);
                    continue;
                }
                if ((dequeue_move_class_mask.0 >> move_class_index.0) & 1) != 0 {
                    let new_value = MoveClassMask(
                        disallowed_move_classes.get(from_state).0 | (1 << move_class_index.0),
                    );
                    disallowed_move_classes.set(from_state, new_value);
                    continue;
                }
                let mut next_state_bits = (dequeue_move_class_mask.0
                    & commutes[move_class_index.0].0)
                    | (1 << move_class_index.0);
                // If a pair of bits are set with the same commutating moves, we
                // can clear out the higher ones. This optimization keeps the
                // state count from going exponential for very big cubes.
                for i in 0..num_move_classes {
                    if (next_state_bits >> i) & 1 != 0 {
                        for j in (i + 1)..num_move_classes {
                            if ((next_state_bits >> j) & 1) != 0 && commutes[i] == commutes[j] {
                                next_state_bits &= !(1 << i);
                            }
                        }
                    }
                }

                let next_move_mask_class = MoveClassMask(next_state_bits);

                next_state[move_class_index.0] = match mask_to_state.get(next_move_mask_class) {
                    None => {
                        let next_state = CanonicalFSMState(state_to_mask.0.len());
                        mask_to_state.insert(next_move_mask_class, next_state);
                        state_to_mask.push(next_move_mask_class);
                        next_state
                    }
                    Some(state) => state,
                };
            }
            next_state_lookup.push(next_state);
        }

        Ok(Self {
            // disallowed_move_classes,
            next_state_lookup,
            // commutes,
            move_class_indices,
        })
    }

    pub(crate) fn next_state(
        &self,
        current_fsm_state: CanonicalFSMState,
        move_class_index: MoveClassIndex,
    ) -> Option<CanonicalFSMState> {
        match self.next_state_lookup[current_fsm_state.0][move_class_index.0] {
            ILLEGAL_FSM_STATE => None,
            state => Some(state),
        }
    }
}
