#[derive(derive_more::From, Debug)]
pub enum CommandError {
    SearchError(PuzzleError),
    ArgumentError(ArgumentError),
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
pub struct PuzzleError {
    pub description: String,
}

impl From<&str> for PuzzleError {
    fn from(description: &str) -> Self {
        Self {
            description: description.to_owned(),
        }
    }
}
