use crate::scramble::{EventError, PuzzleError};

#[derive(derive_more::From, Debug)]
pub enum CommandError {
    ArgumentError(ArgumentError),
    SearchError(SearchError),
    PuzzleError(PuzzleError),
    EventError(EventError),
}

#[derive(Debug)]
pub struct ArgumentError {
    pub description: String,
}

impl From<&str> for ArgumentError {
    fn from(description: &str) -> Self {
        Self {
            description: description.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct SearchError {
    pub description: String,
}

impl From<&str> for SearchError {
    fn from(description: &str) -> Self {
        Self {
            description: description.to_owned(),
        }
    }
}

impl From<&str> for PuzzleError {
    fn from(description: &str) -> Self {
        Self {
            description: description.to_owned(),
        }
    }
}

impl From<&str> for EventError {
    fn from(description: &str) -> Self {
        Self {
            description: description.to_owned(),
        }
    }
}
