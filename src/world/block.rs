use std::fmt::Display;
use std::fmt::{self};
use std::mem;

use crate::visual::light;

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
     const ALL: [Block; Block::BlockCounter as usize] = [
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
     const EMPTY: Block = Block::Air;

     pub fn empty() -> Self
     {
          Self::EMPTY
     }

     pub fn all() -> [Self; Self::BlockCounter as usize]
     {
          Self::ALL
     }

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

     pub fn opacity(&self) -> light::Light
     {
          match self
          {
               | Block::Air => light::Light::new(0),
               | Block::Light => light::Light::new(0),
               | Block::Shrub => light::Light::new(0),
               | Block::Glass => light::Light::new(0),
               | Block::RedFlower => light::Light::new(0),
               | Block::BlueFlower => light::Light::new(0),
               | Block::Leaf => light::Light::new(3),
               | Block::Lava => light::Light::new(5),
               | _ => light::Light::max_light(),
          }
     }

     // pub fn opacity(&self) -> light::Light
     // {
     //      match self
     //      {
     //           | Block::Air => light::Light::new(0),
     //           | Block::Light => light::Light::new(0),
     //           | Block::Shrub => light::Light::new(0),
     //           | Block::Glass => light::Light::new(0),
     //           | Block::RedFlower => light::Light::new(0),
     //           | Block::BlueFlower => light::Light::new(0),
     //           | Block::Water => light::Light::new(3),
     //           | Block::Leaf => light::Light::new(3),
     //           | Block::Lava => light::Light::new(5),
     //           | _ => light::Light::max_light(),
     //      }
     // }

     pub fn visibility(&self) -> Visibility
     {
          match self
          {
               | Block::Air => Visibility::Invisible,
               | Block::Leaf | Block::RedFlower | Block::BlueFlower | Block::Shrub | Block::Glass =>
               {
                    Visibility::PartialOpaque
               }
               | _ => Visibility::Opaque,
          }
     }

     // pub fn visibility(&self) -> Visibility
     // {
     //      match self
     //      {
     //           | Block::Air => Visibility::Invisible,
     //           | Block::Water => Visibility::Transparent,
     //           | Block::Leaf | Block::RedFlower | Block::BlueFlower | Block::Shrub | Block::Glass =>
     //           {
     //                Visibility::PartialOpaque
     //           }
     //           | _ => Visibility::Opaque,
     //      }
     // }

     pub fn emissivity(&self) -> Option<light::Light>
     {
          match self
          {
               | Block::Lava => Some(light::Light::new(8)),
               | Block::Light => Some(light::Light::max_light()),
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
