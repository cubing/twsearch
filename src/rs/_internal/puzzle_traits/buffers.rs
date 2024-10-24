use super::SemiGroupActionPuzzle;

pub struct TransformationBuffer<TPuzzle: SemiGroupActionPuzzle> {
    tpuzzle: TPuzzle,
    a: TPuzzle::Transformation,
    b: TPuzzle::Transformation,
    // In some rough benchmarks, using a boolean to track the current pattern was just a tad faster than using `std::mem::swap(…)`.
    a_is_current: bool,
}

// // TODO: why doesn't this work?
// impl<TPuzzle: SemiGroupActionPuzzle> From<TPuzzle::Transformation>
//     for TransformationBuffer<TPuzzle>
// {
//     fn from(initial: TPuzzle::Transformation) -> Self {
//         Self {
//             b: initial.clone(), // TODO?
//             a: initial,
//             a_is_current: true,
//         }
//     }
// }

impl<TPuzzle: SemiGroupActionPuzzle> TransformationBuffer<TPuzzle> {
    pub fn new(tpuzzle: TPuzzle, initial: TPuzzle::Transformation) -> Self {
        Self {
            tpuzzle,
            b: initial.clone(), // TODO?
            a: initial,
            a_is_current: true,
        }
    }

    pub fn apply_transformation(&mut self, transformation: &TPuzzle::Transformation) {
        if self.a_is_current {
            self.tpuzzle
                .pattern_apply_transformation_into(&self.a, transformation, &mut self.b);
        } else {
            self.b
                .apply_transformation_into(transformation, &mut self.a);
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

impl<TPuzzle: SemiGroupActionPuzzle> PartialEq for TransformationBuffer<TPuzzle> {
    fn eq(&self, other: &Self) -> bool {
        self.current() == other.current()
    }
}
