use crate::_internal::{GenericPuzzle, GenericPuzzleCore};

// TOOD: make this work for patterns using a shared implementation.
pub struct GenericTransformationBuffer<TPuzzle: GenericPuzzleCore> {
    a: TPuzzle::Transformation,
    b: TPuzzle::Transformation,
    // In some rough benchmarks, using a boolean to track the current pattern was just a tad faster than using `std::mem::swap(…)`.
    // TODO: measure this properly across devices, and updated `PackedGenericTransformationBuffer` to match.
    a_is_current: bool,
}

impl<TPuzzle: GenericPuzzle> GenericTransformationBuffer<TPuzzle> {
    pub fn new(initial: TPuzzle::Transformation) -> Self {
        Self {
            b: initial.clone(), // TODO?
            a: initial,
            a_is_current: true,
        }
    }

    pub fn apply_transformation(&mut self, transformation: &TPuzzle::Transformation) {
        if self.a_is_current {
            TPuzzle::transformation_apply_transformation_into(&self.a, transformation, &mut self.b);
        } else {
            TPuzzle::transformation_apply_transformation_into(&self.b, transformation, &mut self.a);
        }
        self.a_is_current = !self.a_is_current
    }

    pub fn current(&self) -> &TPuzzle::Transformation {
        if self.a_is_current {
            &self.a
        } else {
            &self.b
        }
    }
}

impl<TPuzzle: GenericPuzzle> PartialEq for GenericTransformationBuffer<TPuzzle> {
    fn eq(&self, other: &Self) -> bool {
        self.current() == other.current()
    }
}
