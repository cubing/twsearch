pub enum FilteringDecision {
    Accept,
    Reject,
}

impl FilteringDecision {
    // Note: `is_accepted()` might have sounded a bit more intuitive, but `is_accept()` matches Rust conventions better.
    pub fn is_accept(&self) -> bool {
        matches!(self, Self::Accept)
    }

    // Note: `is_rejected()` might have sounded a bit more intuitive, but `is_reject()` matches Rust conventions better.
    pub fn is_reject(&self) -> bool {
        !self.is_accept()
    }
}
