use std::fmt::Display;

/// An error due to the structure of a [`KPattern`] (such as invalid source JSON).
#[derive(Debug)]
pub struct InvalidPatternDataError {
    pub description: String,
}

// TODO: is Rust smart enough to optimize this using just the `From<&str>` Pattern?
impl From<String> for InvalidPatternDataError {
    fn from(description: String) -> Self {
        Self { description }
    }
}

impl From<&str> for InvalidPatternDataError {
    fn from(description: &str) -> Self {
        Self {
            description: description.to_owned(),
        }
    }
}

impl Display for InvalidPatternDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}
