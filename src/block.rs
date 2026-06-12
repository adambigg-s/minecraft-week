use std::{
    fmt::{self, Display},
    mem,
};

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Block {
    #[default]
    Air,
    Grass,
    Sand,
    Water,
    Log,
    Leaf,
    BlockCounter,
}

impl Block {
    pub const ALL: [Block; Block::BlockCounter as usize] = [
        Block::Air,
        Block::Grass,
        Block::Sand,
        Block::Water,
        Block::Log,
        Block::Leaf,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            | Block::Air => "air",
            | Block::Grass => "grass",
            | Block::Sand => "sand",
            | Block::Water => "water",
            | Block::Log => "log",
            | Block::Leaf => "leaf",
            | Block::BlockCounter => "",
        }
    }
}

impl Display for Block {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name())
    }
}

impl<T> From<T> for Block
where
    T: Into<u8>,
{
    fn from(value: T) -> Self {
        unsafe { mem::transmute(value.into()) }
    }
}
