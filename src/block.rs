use std::{
    fmt::{self, Display},
    mem,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility {
    #[default]
    Opaque,
    Transparent,
}

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

        #[rustfmt::skip]
    pub fn name(&self) -> &'static str {
        match self {
            | Block::Air          => "air",
            | Block::Grass        => "grass",
            | Block::Sand         => "sand",
            | Block::Water        => "water",
            | Block::Log          => "log",
            | Block::Leaf         => "leaf",
            | Block::BlockCounter => "",
        }
    }

    pub fn visibility(&self) -> Visibility {
        match self {
            | Block::Leaf | Block::Air => Visibility::Transparent,
            | _ => Visibility::Opaque,
        }
    }

    pub fn random() -> Self {
        Self::from(rand::random_range(1..Block::BlockCounter as u8))
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
