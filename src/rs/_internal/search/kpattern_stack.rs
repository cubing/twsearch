use cubing::kpuzzle::{KPattern, KTransformation};

pub(crate) struct KPatternStack {
    stack: Vec<KPattern>,
    current_idx: usize,
}

impl KPatternStack {
    pub fn new(root_kpattern: KPattern) -> Self {
        Self {
            stack: vec![root_kpattern],
            current_idx: 0,
        }
    }

    pub fn push(&mut self, transformation: &KTransformation) {
        self.current_idx += 1;
        if self.current_idx >= self.stack.len() {
            self.stack
                .push(self.stack[self.current_idx - 1].apply_transformation(transformation))
        } else {
            // We have to use `split_at_mut` so that we can borrow both the read and write entries at the same time: https://doc.rust-lang.org/nomicon/borrow-splitting.html
            let (left, right) = self.stack.split_at_mut(self.current_idx);

            left.last()
                .unwrap()
                .apply_transformation_into(transformation, right.first_mut().unwrap());
        }
    }

    pub fn current_pattern(&self) -> &KPattern {
        &self.stack[self.current_idx]
    }

    pub fn pop(&mut self) {
        self.current_idx -= 1;
    }
}
