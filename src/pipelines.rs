use crate::render;

pub struct Rainbow;
impl render::GfxPipeline for Rainbow {
    fn pipeline(
        gfx_context: &render::GfxContext,
        gfx_layouts: &[Option<&wgpu::BindGroupLayout>],
    ) -> wgpu::RenderPipeline {
        todo!()
    }
}
