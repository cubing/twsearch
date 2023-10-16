use crate::_internal::SearchError;

// TODO: move this to another export location.
#[derive(Debug)]
pub enum Event {
    Cube2x2x2,
    Cube3x3x3,
    Pyraminx,
}

impl TryFrom<&str> for Event {
    type Error = SearchError;

    fn try_from(event_str: &str) -> Result<Self, Self::Error> {
        Ok(match event_str {
            "222" => Event::Cube2x2x2,
            "333" => Event::Cube3x3x3,
            "pyram" => Event::Pyraminx,
            _ => {
                return Err(SearchError {
                    description: format!("Unknown event ID: {}", event_str),
                })
            }
        })
    }
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
