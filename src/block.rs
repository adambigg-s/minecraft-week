use std::fmt::{self, Display};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Block {
    #[default]
    Air,
    Grass,
    Sand,
    Water,
    Log,
    Leaf,
}

impl Block {
    pub fn name(&self) -> &'static str {
        match self {
            | Block::Air => "air",
            | Block::Grass => "grass",
            | Block::Sand => "sand",
            | Block::Water => "water",
            | Block::Log => "log",
            | Block::Leaf => "leaf",
        }
    }
}

impl Display for Block {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name())
    }
}
