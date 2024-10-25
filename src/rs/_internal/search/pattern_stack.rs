use crate::_internal::puzzle_traits::SemiGroupActionPuzzle;

pub(crate) struct PatternStack<TPuzzle: SemiGroupActionPuzzle> {
    puzzle: TPuzzle,
    stack: Vec<TPuzzle::Pattern>,
    current_idx: usize,
}

impl<TPuzzle: SemiGroupActionPuzzle> PatternStack<TPuzzle> {
    pub fn new(puzzle: TPuzzle, root_pattern: TPuzzle::Pattern) -> Self {
        Self {
            puzzle,
            stack: vec![root_pattern],
            current_idx: 0,
        }
    }

    pub fn push(&mut self, transformation: &TPuzzle::Transformation) {
        self.current_idx += 1;
        if self.current_idx >= self.stack.len() {
            self.stack.push(
                self.puzzle.pattern_apply_transformation(
                    &self.stack[self.current_idx - 1],
                    transformation,
                ),
            )
        } else {
            // We have to use `split_at_mut` so that we can borrow both the read and write entries at the same time: https://doc.rust-lang.org/nomicon/borrow-splitting.html
            let (left, right) = self.stack.split_at_mut(self.current_idx);

            self.puzzle.pattern_apply_transformation_into(
                left.last().unwrap(),
                transformation,
                right.first_mut().unwrap(),
            );
        }
    }

    pub fn current_pattern(&self) -> &TPuzzle::Pattern {
        &self.stack[self.current_idx]
    }

    // Note: this function does not perform any bound checking in release mode.
    // Calling code must ensure it never calls calls `.pop()`` more often than
    // it has called `.push(…)`.
    pub fn pop(&mut self) {
        debug_assert!(self.current_idx > 0);
        self.current_idx -= 1;
    }
}
