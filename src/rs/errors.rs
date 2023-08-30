#[derive(derive_more::From, Debug)]
pub enum CommandError {
    SearchError(SearchError),
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
