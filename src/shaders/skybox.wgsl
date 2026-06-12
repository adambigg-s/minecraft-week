const EPS: f32 = 1e-3;

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) tex: vec2<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex: vec2<f32>,
};

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    var view_proj = view_proj;
    view_proj[3] = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.pos = view_proj * vec4<f32>(in.pos, 1.0);
    out.tex = in.tex;
    return out;
}

@group(0) @binding(1) var texture_atlas: texture_2d<f32>;
@group(0) @binding(2) var sample_atlas: sampler;

@group(1) @binding(0) var texture_skybox: texture_2d<f32>;
@group(1) @binding(1) var sampler_skybox: sampler;

@fragment
fn fs_main(out: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(texture_skybox, sampler_skybox, out.tex);
}

