const EPS: f32 = 1e-3;

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex: vec2<f32>,
    @location(3) fil: f32,
    @location(4) bil: f32,
    @location(5) ao: f32,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex: vec2<f32>,
    @location(1) fil: f32,
    @location(2) bil: f32,
}

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.pos = view_proj * vec4<f32>(in.pos, 1.0);
    out.tex = in.tex;
    out.fil = in.fil;
    out.bil = in.bil;
    return out;
}

@group(0) @binding(2) var texture_atlas: texture_2d<f32>;
@group(0) @binding(3) var sample_atlas: sampler;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let color = textureSample(texture_atlas, sample_atlas, in.tex);
    let light = mix(vec4<f32>(in.bil), vec4<f32>(in.fil, 0.0, 0.0, 0.0), 0.25);
    if color.a < EPS {
        discard;
    }

    return mix(color, light, 0.95);
}

