use super::Event;

pub struct PuzzleError {
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Puzzle {
    Cube3x3x3,
    Cube2x2x2,
    Cube4x4x4,
    Cube5x5x5,
    Cube6x6x6,
    Cube7x7x7,
    Clock,
    Megaminx,
    Pyraminx,
    Skewb,
    Square1,
    FTO,
    MasterTetraminx,
    Kilominx,
    RediCube,
}

impl Puzzle {
    pub fn id(&self) -> &str {
        match self {
            Self::Cube3x3x3 => "3x3x3",
            Self::Cube2x2x2 => "2x2x2",
            Self::Cube4x4x4 => "4x4x4",
            Self::Cube5x5x5 => "5x5x5",
            Self::Cube6x6x6 => "6x6x6",
            Self::Cube7x7x7 => "7x7x7",
            Self::Clock => "clock",
            Self::Megaminx => "megaminx",
            Self::Pyraminx => "pyraminx",
            Self::Skewb => "skewb",
            Self::Square1 => "square1",
            Self::FTO => "fto",
            Self::MasterTetraminx => "master_tetraminx",
            Self::Kilominx => "kilominx",
            Self::RediCube => "redi_cube",
        }
    }

    pub fn try_from_id(puzzle_id_str: &str) -> Result<Self, PuzzleError> {
        Ok(match puzzle_id_str {
            "3x3x3" => Self::Cube3x3x3,
            "2x2x2" => Self::Cube2x2x2,
            "4x4x4" => Self::Cube4x4x4,
            "5x5x5" => Self::Cube5x5x5,
            "6x6x6" => Self::Cube6x6x6,
            "7x7x7" => Self::Cube7x7x7,
            "clock" => Self::Clock,
            "megaminx" => Self::Megaminx,
            "pyraminx" => Self::Pyraminx,
            "skewb" => Self::Skewb,
            "square1" => Self::Square1,
            "fto" => Self::FTO,
            "master_tetraminx" => Self::MasterTetraminx,
            "kilominx" => Self::Kilominx,
            "redi_cube" => Self::RediCube,
            _ => {
                return Err(PuzzleError {
                    description: format!("Unknown puzzle ID: {}", puzzle_id_str),
                })
            }
        })
    }

    pub fn speedsolving_event(&self) -> Event {
        match self {
            Self::Cube3x3x3 => Event::Cube3x3x3Speedsolving,
            Self::Cube2x2x2 => Event::Cube2x2x2Speedsolving,
            Self::Cube4x4x4 => Event::Cube4x4x4Speedsolving,
            Self::Cube5x5x5 => Event::Cube5x5x5Speedsolving,
            Self::Cube6x6x6 => Event::Cube6x6x6Speedsolving,
            Self::Cube7x7x7 => Event::Cube7x7x7Speedsolving,
            Self::Clock => Event::ClockSpeedsolving,
            Self::Megaminx => Event::MegaminxSpeedsolving,
            Self::Pyraminx => Event::PyraminxSpeedsolving,
            Self::Skewb => Event::SkewbSpeedsolving,
            Self::Square1 => Event::Square1Speedsolving,
            Self::FTO => Event::FTOSpeedsolving,
            Self::MasterTetraminx => Event::MasterTetraminxSpeedsolving,
            Self::Kilominx => Event::KilominxSpeedsolving,
            Self::RediCube => Event::RediCubeSpeedsolving,
        }
    }
}
