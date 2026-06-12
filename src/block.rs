use std::fmt::{self, Display};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Block {
    #[default]
    Air,
    Grass,
}

impl Block {
    pub fn name(&self) -> &'static str {
        match self {
            | Block::Air => "air",
            | Block::Grass => "grass",
        }
    }
}

impl Display for Block {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name())
    }
}
