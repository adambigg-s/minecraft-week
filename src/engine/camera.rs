use std::{
    f32,
    fmt::{self, Display},
};

use crate::{engine::transform, render};

#[derive(bon::Builder, Debug, Default)]

pub struct Camera {
    #[builder(default)]
    pub inner: transform::Transform,
    #[builder(default)]
    pub ar: f32,
    pub fov: f32,
    pub znear: f32,
    pub zfear: f32,
    #[builder(default)]
    pub pitch: f32,
    #[builder(default)]
    pub yaw: f32,
}

impl Camera {
    pub fn update_rotation(&mut self, dx: f32, dy: f32, dz: f32) {
        self.inner.rotation *= glam::Quat::from_rotation_y(dy)
            * glam::Quat::from_rotation_x(dx)
            * glam::Quat::from_rotation_z(dz);
    }

    pub fn update_position(&mut self, dx: f32, dy: f32, dz: f32) {
        self.inner.position += self.inner.forward() * dz;
        self.inner.position += self.inner.right() * dx;
        self.inner.position += self.inner.up() * dy;
    }

    pub fn confine_euler(&mut self) {
        self.pitch = self.pitch.clamp(-f32::consts::PI / 2.0 * 0.99, f32::consts::PI / 2.0 * 0.99);
        self.yaw %= f32::consts::TAU;
    }

    pub fn view(&self) -> glam::Mat4 {
        self.inner.to_matrix4().inverse()
    }

    pub fn proj(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fov.to_radians(), self.ar, self.znear, self.zfear)
    }
}

impl render::GfxCamera for Camera {
    fn view_proj(&self) -> glam::Mat4 {
        self.proj() * self.view()
    }
}

impl Display for Camera {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "Camera pos: {:.1}", self.inner.position)?;
        writeln!(fmt, "Camera fvec: {:.2}", self.inner.forward())?;
        writeln!(fmt, "Camera rvec: {:.2}", self.inner.right())?;
        writeln!(fmt, "Camera uvec: {:.2}", self.inner.up())?;
        Ok(())
    }
}
