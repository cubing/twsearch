pub enum Event {
    Cube2x2x2,
    Cube3x3x3,
    Pyraminx,
}

impl Event {
    pub fn id(&self) -> &str {
        match self {
            Event::Cube2x2x2 => "222",
            Event::Cube3x3x3 => "333",
            Event::Pyraminx => "pyram",
        }
    }
}
