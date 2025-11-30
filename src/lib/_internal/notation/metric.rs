use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum TurnMetric {
    #[default]
    Hand,
    Quantum,
}

impl Display for TurnMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TurnMetric::Hand => "hand",
            TurnMetric::Quantum => "quantum",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for TurnMetric {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "hand" => Self::Hand,
            "quantum" => Self::Quantum,
            _ => Err("Invalid turn metric".to_owned())?,
        })
    }
}
