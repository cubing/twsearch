// TODO: move all this to cubing.js

use std::fmt::Display;

use super::Puzzle;

#[derive(Debug)]
pub struct EventError {
    pub description: String,
}

// TODO: move this to another export location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Cube3x3x3Speedsolving,
    Cube2x2x2Speedsolving,
    Cube4x4x4Speedsolving,
    Cube5x5x5Speedsolving,
    Cube6x6x6Speedsolving,
    Cube7x7x7Speedsolving,
    Cube3x3x3Blindfolded,
    Cube3x3x3FewestMoves,
    Cube3x3x3OneHanded,
    ClockSpeedsolving,
    MegaminxSpeedsolving,
    PyraminxSpeedsolving,
    SkewbSpeedsolving,
    Square1Speedsolving,
    Cube4x4x4Blindfolded,
    Cube5x5x5Blindfolded,
    Cube3x3x3MultiBlind,
    FTOSpeedsolving,
    MasterTetraminxSpeedsolving,
    KilominxSpeedsolving,
    RediCubeSpeedsolving,
    BabyFTOSpeedsolving,
}

impl TryFrom<&str> for Event {
    type Error = EventError;

    fn try_from(event_str: &str) -> Result<Self, Self::Error> {
        Ok(match event_str {
            "333" => Self::Cube3x3x3Speedsolving,
            "222" => Self::Cube2x2x2Speedsolving,
            "444" => Self::Cube4x4x4Speedsolving,
            "555" => Self::Cube5x5x5Speedsolving,
            "666" => Self::Cube6x6x6Speedsolving,
            "777" => Self::Cube7x7x7Speedsolving,
            "333bf" => Self::Cube3x3x3Blindfolded,
            "333fm" => Self::Cube3x3x3FewestMoves,
            "333oh" => Self::Cube3x3x3OneHanded,
            "clock" => Self::ClockSpeedsolving,
            "minx" => Self::MegaminxSpeedsolving,
            "pyram" => Self::PyraminxSpeedsolving,
            "skewb" => Self::SkewbSpeedsolving,
            "sq1" => Self::Square1Speedsolving,
            "444bf" => Self::Cube4x4x4Blindfolded,
            "555bf" => Self::Cube5x5x5Blindfolded,
            "333mbf" => Self::Cube3x3x3MultiBlind,
            "fto" => Self::FTOSpeedsolving,
            "master_tetraminx" => Self::MasterTetraminxSpeedsolving,
            "kilominx" => Self::KilominxSpeedsolving,
            "redi_cube" => Self::RediCubeSpeedsolving,
            "baby_fto" => Self::BabyFTOSpeedsolving,
            _ => {
                return Err(EventError {
                    description: format!("Unknown event ID: {}", event_str),
                })
            }
        })
    }
}

impl Event {
    pub fn id(&self) -> &str {
        match self {
            Self::Cube3x3x3Speedsolving => "333",
            Self::Cube2x2x2Speedsolving => "222",
            Self::Cube4x4x4Speedsolving => "444",
            Self::Cube5x5x5Speedsolving => "555",
            Self::Cube6x6x6Speedsolving => "666",
            Self::Cube7x7x7Speedsolving => "777",
            Self::Cube3x3x3Blindfolded => "333bf",
            Self::Cube3x3x3FewestMoves => "333fm",
            Self::Cube3x3x3OneHanded => "333oh",
            Self::ClockSpeedsolving => "clock",
            Self::MegaminxSpeedsolving => "minx",
            Self::PyraminxSpeedsolving => "pyram",
            Self::SkewbSpeedsolving => "skewb",
            Self::Square1Speedsolving => "sq1",
            Self::Cube4x4x4Blindfolded => "444bf",
            Self::Cube5x5x5Blindfolded => "555bf",
            Self::Cube3x3x3MultiBlind => "333mbf",
            Self::FTOSpeedsolving => "fto",
            Self::MasterTetraminxSpeedsolving => "master_tetraminx",
            Self::KilominxSpeedsolving => "kilominx",
            Self::RediCubeSpeedsolving => "redi_cube",
            Self::BabyFTOSpeedsolving => "baby_fto",
        }
    }

    pub fn puzzle(&self) -> Puzzle {
        match self {
            Self::Cube3x3x3Speedsolving => Puzzle::Cube3x3x3,
            Self::Cube2x2x2Speedsolving => Puzzle::Cube2x2x2,
            Self::Cube4x4x4Speedsolving => Puzzle::Cube4x4x4,
            Self::Cube5x5x5Speedsolving => Puzzle::Cube5x5x5,
            Self::Cube6x6x6Speedsolving => Puzzle::Cube6x6x6,
            Self::Cube7x7x7Speedsolving => Puzzle::Cube7x7x7,
            Self::Cube3x3x3Blindfolded => Puzzle::Cube3x3x3,
            Self::Cube3x3x3FewestMoves => Puzzle::Cube3x3x3,
            Self::Cube3x3x3OneHanded => Puzzle::Cube3x3x3,
            Self::ClockSpeedsolving => Puzzle::Clock,
            Self::MegaminxSpeedsolving => Puzzle::Megaminx,
            Self::PyraminxSpeedsolving => Puzzle::Pyraminx,
            Self::SkewbSpeedsolving => Puzzle::Skewb,
            Self::Square1Speedsolving => Puzzle::Square1,
            Self::Cube4x4x4Blindfolded => Puzzle::Cube4x4x4,
            Self::Cube5x5x5Blindfolded => Puzzle::Cube5x5x5,
            Self::Cube3x3x3MultiBlind => Puzzle::Cube3x3x3,
            Self::FTOSpeedsolving => Puzzle::FTO,
            Self::MasterTetraminxSpeedsolving => Puzzle::MasterTetraminx,
            Self::KilominxSpeedsolving => Puzzle::Kilominx,
            Self::RediCubeSpeedsolving => Puzzle::RediCube,
            Self::BabyFTOSpeedsolving => Puzzle::BabyFTO,
        }
    }

    pub fn event_name(&self) -> &str {
        match self {
            Self::Cube3x3x3Speedsolving => "3x3x3 Cube",
            Self::Cube2x2x2Speedsolving => "2x2x2 Cube",
            Self::Cube4x4x4Speedsolving => "4x4x4 Cube",
            Self::Cube5x5x5Speedsolving => "5x5x5 Cube",
            Self::Cube6x6x6Speedsolving => "6x6x6 Cube",
            Self::Cube7x7x7Speedsolving => "7x7x7 Cube",
            Self::Cube3x3x3Blindfolded => "3x3x3 Blindfolded",
            Self::Cube3x3x3FewestMoves => "3x3x3 Fewest Moves",
            Self::Cube3x3x3OneHanded => "3x3x3 One-Handed",
            Self::ClockSpeedsolving => "Clock",
            Self::MegaminxSpeedsolving => "Megaminx",
            Self::PyraminxSpeedsolving => "Pyraminx",
            Self::SkewbSpeedsolving => "Skewb",
            Self::Square1Speedsolving => "Square-1",
            Self::Cube4x4x4Blindfolded => "4x4x4 Blindfolded",
            Self::Cube5x5x5Blindfolded => "5x5x5 Blindfolded",
            Self::Cube3x3x3MultiBlind => "3x3x3 Multi-Blind",
            Self::FTOSpeedsolving => "Face-Turning Octahedron",
            Self::MasterTetraminxSpeedsolving => "Master Tetraminx",
            Self::KilominxSpeedsolving => "Kilominx",
            Self::RediCubeSpeedsolving => "Redi Cube",
            Self::BabyFTOSpeedsolving => "Baby FTO",
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}
