pub enum FilteringDecision {
    Accept,
    Reject,
}

impl FilteringDecision {
    pub fn is_accept(&self) -> bool {
        matches!(self, Self::Accept)
    }

    pub fn is_reject(&self) -> bool {
        !self.is_accept()
    }
}
