use crate::_internal::GenericPuzzleCore;

// TOOD: make this work for patterns using a shared implementation.
pub struct GenericPatternBuffer<TPuzzle: GenericPuzzleCore> {
    a: TPuzzle::Pattern,
    b: TPuzzle::Pattern,
    // In some rough benchmarks, using a boolean to track the current pattern was just a tad faster than using `std::mem::swap(…)`.
    // TODO: measure this properly across devices, and updated `PackedGenericPatternBuffer` to match.
    a_is_current: bool,
}

impl<TPuzzle: GenericPuzzleCore> GenericPatternBuffer<TPuzzle> {
    pub fn new(initial: TPuzzle::Pattern) -> Self {
        Self {
            b: initial.clone(), // TODO?
            a: initial,
            a_is_current: true,
        }
    }

    pub fn apply_transformation(&mut self, transformation: &TPuzzle::Transformation) {
        if self.a_is_current {
            TPuzzle::pattern_apply_transformation_into(&self.a, transformation, &mut self.b);
        } else {
            TPuzzle::pattern_apply_transformation_into(&self.b, transformation, &mut self.a);
        }
        self.a_is_current = !self.a_is_current
    }

    pub fn current(&self) -> &TPuzzle::Pattern {
        if self.a_is_current {
            &self.a
        } else {
            &self.b
        }
    }
}

impl<TPuzzle: GenericPuzzleCore> PartialEq for GenericPatternBuffer<TPuzzle> {
    fn eq(&self, other: &Self) -> bool {
        self.current() == other.current()
    }
}
