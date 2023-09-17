use std::ops::BitAndAssign;

use cubing::alg::Move;
use twsearch::PackedKTransformation;

// Change the appropriate types to use another type if this is updated.
const MAX_NUM_MOVE_CLASSES: usize = 64;

#[derive(Debug)]
struct MoveClass(usize);

// Bit N is indexed by a `MoveClass` value of N.
#[derive(Debug)]
struct MoveClassMask(u64);

impl BitAndAssign for MoveClassMask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 = self.0 & rhs.0
    }
}

#[derive(Debug)]
pub struct MoveInfo {
    pub(crate) r#move: Move,
    // move_class: MoveClass, // TODO: do we need this?
    pub(crate) metric_turns: i32,
    pub(crate) transformation: PackedKTransformation,
    pub(crate) inverse_transformation: PackedKTransformation,
}

#[derive(Debug)]
pub struct AllMoveMultiples {
    // Indexed by `MoveClass`, then `amount`
    pub(crate) multiples: Vec<Vec<MoveInfo>>,
}

fn do_transformations_commute(t1: &PackedKTransformation, t2: &PackedKTransformation) -> bool {
    t1.apply_transformation(t2) == t2.apply_transformation(t1)
}

#[derive(Debug)]
struct CanonicalFSMState(u64);

#[derive(Debug)]
pub struct CanonicalFSM {
    commutes: Vec<MoveClassMask>,
}

impl CanonicalFSM {
    // TODO: Return a more specific error.
    pub fn try_new(all_move_multiples: AllMoveMultiples) -> Result<CanonicalFSM, String> {
        let num_move_classes = all_move_multiples.multiples.len();
        if num_move_classes > MAX_NUM_MOVE_CLASSES {
            return Err("Too many move classes!".to_owned());
        }

        let mut commutes: Vec<MoveClassMask> = Vec::new();
        for i in 0..num_move_classes {
            commutes.push(MoveClassMask((1 << num_move_classes) - 1))
        }

        // written this way so if we later iterate over all moves instead of
        // all move classes, because perhaps the first move commutes and later
        // ones don't, we maintain the same logic.
        for i in 0..num_move_classes {
            for j in 0..num_move_classes {
                if do_transformations_commute(
                    &all_move_multiples.multiples[i][0].transformation,
                    &all_move_multiples.multiples[j][0].transformation,
                ) {
                    commutes[i] &= MoveClassMask(!(1 << j));
                    commutes[j] &= MoveClassMask(!(1 << i));
                }
            }
        }

        Ok(Self { commutes })
    }

    // fn initial_state(&self) -> CanonicalFSMState {
    //     CanonicalFSMState(0)
    // }
    // fn move_mask(&self, state: CanonicalFSMState) -> MoveClassMask {
    //     MoveClassMask(0)
    // }
    // fn nextState(
    //     &self,
    //     state: CanonicalFSMState,
    //     moveClass: CanonicalFSMMoveClass,
    // ) -> CanonicalFSMState {
    //     nextStateVec[state][moveClass]
    // }
}
