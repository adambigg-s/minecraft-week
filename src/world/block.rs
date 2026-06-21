use std::fmt::Display;
use std::fmt::{self};
use std::mem;

use crate::engine::kinematics;
use crate::world::light;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility
{
     #[default]
     Invisible,
     Opaque,
     PartialOpaque,
     Transparent,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EmittedMesh
{
     #[default]
     RectilinearFull,
     RectilinearPartial,
     Decorator,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Block
{
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
     BlueFlower,
     Shrub,
     Coal,
     Copper,
     Tin,
     Glass,
     Light,
     BlockCounter,
}

impl Block
{
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
          Block::BlueFlower,
          Block::Shrub,
          Block::Coal,
          Block::Copper,
          Block::Tin,
          Block::Glass,
          Block::Light,
     ];
     pub const EMPTY: Block = Block::Air;

     pub fn name(&self) -> &'static str
     {
          match self
          {
               | Block::Air => "air",
               | Block::Dirt => "dirt",
               | Block::Grass => "grass",
               | Block::Sand => "sand",
               | Block::Water => "water",
               | Block::Lava => "lava",
               | Block::Log => "log",
               | Block::Leaf => "leaf",
               | Block::Stone => "stone",
               | Block::Gravel => "gravel",
               | Block::Plank => "plank",
               | Block::Quartz => "quartz",
               | Block::RedFlower => "redflower",
               | Block::BlueFlower => "blueflower",
               | Block::Shrub => "shrub",
               | Block::Coal => "coal",
               | Block::Copper => "copper",
               | Block::Tin => "tin",
               | Block::Glass => "glass",
               | Block::Light => "light",
               | Block::BlockCounter => "",
          }
     }

     pub fn opacity(&self) -> u8
     {
          match self
          {
               | Block::Air => 0,
               | Block::Light => 0,
               | Block::Shrub => 0,
               | Block::Glass => 0,
               | Block::RedFlower => 0,
               | Block::BlueFlower => 0,
               | Block::Water => 3,
               | Block::Leaf => 3,
               | Block::Lava => 5,
               | _ => light::MAX_LIGHT,
          }
     }

     pub fn visibility(&self) -> Visibility
     {
          match self
          {
               | Block::Air => Visibility::Invisible,
               | Block::Water => Visibility::Transparent,
               | Block::Leaf | Block::RedFlower | Block::BlueFlower | Block::Shrub | Block::Glass =>
               {
                    Visibility::PartialOpaque
               }
               | _ => Visibility::Opaque,
          }
     }

     pub fn emissivity(&self) -> Option<u8>
     {
          match self
          {
               | Block::Lava => Some(8),
               | Block::Light => Some(light::MAX_LIGHT),
               | _ => None,
          }
     }

     pub fn mesh_style(&self) -> EmittedMesh
     {
          match self
          {
               | Block::RedFlower | Block::BlueFlower | Block::Shrub => EmittedMesh::Decorator,
               | _ => EmittedMesh::RectilinearFull,
          }
     }

     pub fn random() -> Self
     {
          Self::ALL[rand::random_range(0 .. Self::BlockCounter as u8) as usize]
     }
}

impl Display for Block
{
     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result
     {
          write!(fmt, "{}", self.name())
     }
}

impl<T> From<T> for Block
where
     T: Into<u8>,
{
     fn from(value: T) -> Self
     {
          unsafe { mem::transmute(value.into()) }
     }
}

impl kinematics::Collision for Block
{
     type Collider = ();

     fn collides(&self, _: Self::Collider) -> bool
     {
          match self
          {
               | Block::Air
               | Block::Water
               | Block::RedFlower
               | Block::BlueFlower
               | Block::Shrub
               | Block::Lava => false,
               | _ => true,
          }
     }
}
