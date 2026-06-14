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
    Dirt,
    Grass,
    Sand,
    Water,
    Lava,
    Log,
    Leaf,
    Stone,
    Gravel,
    Plank,
    Quartz,
    RedFlower,
    BlockCounter,
}

impl Block {
    pub const ALL: [Block; Block::BlockCounter as usize] = [
        Block::Air,
        Block::Dirt,
        Block::Grass,
        Block::Sand,
        Block::Water,
        Block::Lava,
        Block::Log,
        Block::Leaf,
        Block::Stone,
        Block::Gravel,
        Block::Plank,
        Block::Quartz,
        Block::RedFlower,
    ];

    #[rustfmt::skip]
    pub fn name(&self) -> &'static str {
        match self {
            | Block::Air          => "air",
            | Block::Dirt         => "dirt",
            | Block::Grass        => "grass",
            | Block::Sand         => "sand",
            | Block::Water        => "water",
            | Block::Lava         => "lava",
            | Block::Log          => "log",
            | Block::Leaf         => "leaf",
            | Block::Stone        => "stone",
            | Block::Gravel       => "gravel",
            | Block::Plank        => "plank",
            | Block::Quartz       => "quartz",
            | Block::RedFlower    => "redflower",
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
        Self::ALL[rand::random_range(0..Self::BlockCounter as u8) as usize]
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
