#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Face
{
     #[default]
     Top,
     Bottom,
     Left,
     Right,
     Back,
     Front,
     // DiagPosFront,
     // DiagPosBack,
     // DiagNegFront,
     // DiagNegBack,
}

impl Face
{
     const CARDINALS: [Face; 6] = [
          Face::Top,
          Face::Bottom,
          Face::Left,
          Face::Right,
          Face::Front,
          Face::Back,
     ];
     // pub const DIAGONALS: [Face; 4] = [
     //      Face::DiagPosFront,
     //      Face::DiagPosBack,
     //      Face::DiagNegFront,
     //      Face::DiagNegBack,
     // ];

     pub fn cardinals() -> [Self; 6]
     {
          Self::CARDINALS
     }

     pub fn neighbor_offset(&self) -> glam::IVec3
     {
          match self
          {
               | Face::Top => glam::ivec3(0, 1, 0),
               | Face::Bottom => glam::ivec3(0, -1, 0),
               | Face::Left => glam::ivec3(-1, 0, 0),
               | Face::Right => glam::ivec3(1, 0, 0),
               | Face::Back => glam::ivec3(0, 0, 1),
               | Face::Front => glam::ivec3(0, 0, -1),
               // | _ => glam::IVec3::ZERO,
          }
     }

     pub fn normal(&self) -> glam::Vec3
     {
          match self
          {
               | Face::Top => glam::Vec3::Y,
               | Face::Bottom => glam::Vec3::NEG_Y,
               | Face::Left => glam::Vec3::X,
               | Face::Right => glam::Vec3::NEG_X,
               | Face::Back => glam::Vec3::Z,
               | Face::Front => glam::Vec3::NEG_Z,
               // | Face::DiagPosFront => glam::ivec3(1, 0, -1).as_vec3().normalize(),
               // | Face::DiagPosBack => glam::ivec3(1, 0, 1).as_vec3().normalize(),
               // | Face::DiagNegFront => glam::ivec3(-1, 0, -1).as_vec3().normalize(),
               // | Face::DiagNegBack => glam::ivec3(-1, 0, 1).as_vec3().normalize(),
          }
     }

     pub fn corners(&self) -> [(glam::IVec3, glam::IVec2); 4]
     {
          use glam::ivec2;
          use glam::ivec3;

          match self
          {
               | Face::Top =>
               {
                    [
                         (ivec3(0, 1, 1), ivec2(0, 0)),
                         (ivec3(0, 1, 0), ivec2(0, 1)),
                         (ivec3(1, 1, 1), ivec2(1, 0)),
                         (ivec3(1, 1, 0), ivec2(1, 1)),
                    ]
               }
               | Face::Bottom =>
               {
                    [
                         (ivec3(0, 0, 0), ivec2(0, 0)),
                         (ivec3(0, 0, 1), ivec2(0, 1)),
                         (ivec3(1, 0, 0), ivec2(1, 0)),
                         (ivec3(1, 0, 1), ivec2(1, 1)),
                    ]
               }
               | Face::Left =>
               {
                    [
                         (ivec3(0, 1, 1), ivec2(0, 0)),
                         (ivec3(0, 0, 1), ivec2(0, 1)),
                         (ivec3(0, 1, 0), ivec2(1, 0)),
                         (ivec3(0, 0, 0), ivec2(1, 1)),
                    ]
               }
               | Face::Right =>
               {
                    [
                         (ivec3(1, 1, 0), ivec2(0, 0)),
                         (ivec3(1, 0, 0), ivec2(0, 1)),
                         (ivec3(1, 1, 1), ivec2(1, 0)),
                         (ivec3(1, 0, 1), ivec2(1, 1)),
                    ]
               }
               | Face::Back =>
               {
                    [
                         (ivec3(1, 1, 1), ivec2(0, 0)),
                         (ivec3(1, 0, 1), ivec2(0, 1)),
                         (ivec3(0, 1, 1), ivec2(1, 0)),
                         (ivec3(0, 0, 1), ivec2(1, 1)),
                    ]
               }
               | Face::Front =>
               {
                    [
                         (ivec3(0, 1, 0), ivec2(0, 0)),
                         (ivec3(0, 0, 0), ivec2(0, 1)),
                         (ivec3(1, 1, 0), ivec2(1, 0)),
                         (ivec3(1, 0, 0), ivec2(1, 1)),
                    ]
               } // | Face::DiagPosFront =>
                 // {
                 //      [
                 //           (ivec3(0, 1, 0), ivec2(0, 0)),
                 //           (ivec3(0, 0, 0), ivec2(0, 1)),
                 //           (ivec3(1, 1, 1), ivec2(1, 0)),
                 //           (ivec3(1, 0, 1), ivec2(1, 1)),
                 //      ]
                 // }
                 // | Face::DiagPosBack =>
                 // {
                 //      [
                 //           (ivec3(1, 1, 1), ivec2(0, 0)),
                 //           (ivec3(1, 0, 1), ivec2(0, 1)),
                 //           (ivec3(0, 1, 0), ivec2(1, 0)),
                 //           (ivec3(0, 0, 0), ivec2(1, 1)),
                 //      ]
                 // }
                 // | Face::DiagNegFront =>
                 // {
                 //      [
                 //           (ivec3(1, 1, 0), ivec2(0, 0)),
                 //           (ivec3(1, 0, 0), ivec2(0, 1)),
                 //           (ivec3(0, 1, 1), ivec2(1, 0)),
                 //           (ivec3(0, 0, 1), ivec2(1, 1)),
                 //      ]
                 // }
                 // | Face::DiagNegBack =>
                 // {
                 //      [
                 //           (ivec3(0, 1, 1), ivec2(0, 0)),
                 //           (ivec3(0, 0, 1), ivec2(0, 1)),
                 //           (ivec3(1, 1, 0), ivec2(1, 0)),
                 //           (ivec3(1, 0, 0), ivec2(1, 1)),
                 //      ]
                 // }
          }
     }
}

#[derive(bon::Builder, Debug, Default)]
pub struct Quad
{
     pub position: glam::IVec3,
     pub face: Face,
}

impl Quad
{
     pub fn cube() -> [Self; 6]
     {
          Face::CARDINALS.map(|face| {
               Self {
                    position: glam::IVec3::ZERO,
                    face,
               }
          })
     }

     pub fn positions(&self) -> [glam::Vec3; 4]
     {
          self.face.corners().map(|(offset, _)| (self.position + offset).as_vec3())
     }

     pub fn texture_uvs(&self) -> [glam::Vec2; 4]
     {
          self.face.corners().map(|(_, uv)| uv.as_vec2())
     }

     pub fn normals(&self) -> [glam::Vec3; 4]
     {
          [self.face.normal(); 4]
     }

     pub fn indices(&self, start: u32) -> [u32; 6]
     {
          [start, start + 2, start + 1, start + 1, start + 2, start + 3]
     }
}

#[derive(bon::Builder, Debug)]
pub struct RectilinearMeshSlice<'r>
{
     pub face: Face,
     pub integer_position: glam::IVec3,
     pub pos: &'r mut [glam::Vec3],
     pub nor: &'r mut [glam::Vec3],
     pub uvs: &'r mut [glam::Vec2],
}

#[derive(bon::Builder, Debug, Default)]
pub struct RectilinearMesh
{
     pub pos: Vec<glam::Vec3>,
     pub nor: Vec<glam::Vec3>,
     pub uvs: Vec<glam::Vec2>,
     pub index: Vec<u32>,
     pub integer_pos: Vec<glam::IVec3>,
     pub face: Vec<Face>,
     pub size: usize,
}

impl RectilinearMesh
{
     pub fn from_quads(quads: &[Quad]) -> Self
     {
          let mut out = Self {
               size: quads.len(),
               ..Default::default()
          };
          quads.iter().for_each(|quad| {
               out.index.extend_from_slice(&quad.indices(out.pos.len() as u32));
               out.pos.extend_from_slice(&quad.positions());
               out.nor.extend_from_slice(&quad.normals());
               out.uvs.extend_from_slice(&quad.texture_uvs());
               out.face.push(quad.face);
               out.integer_pos.push(quad.position);
          });
          out
     }

     pub fn quad_slice<'r>(&'r mut self, index: usize) -> RectilinearMeshSlice<'r>
     {
          let offset = index * 4;
          RectilinearMeshSlice {
               face: self.face[index],
               integer_position: self.integer_pos[index],
               pos: &mut self.pos[offset .. offset + 4],
               nor: &mut self.nor[offset .. offset + 4],
               uvs: &mut self.uvs[offset .. offset + 4],
          }
     }

     pub fn unit_cube() -> Self
     {
          Self::from_quads(&Quad::cube())
     }

     pub fn scale(&mut self, scale: glam::Vec3)
     {
          self.pos.iter_mut().for_each(|pos| {
               *pos *= scale;
          });
     }

     pub fn shift(&mut self, shift: glam::Vec3)
     {
          self.pos.iter_mut().for_each(|pos| {
               *pos += shift;
          });
     }
}
