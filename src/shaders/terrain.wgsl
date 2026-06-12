const EPS: f32 = 1e-3;

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex: vec2<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) nor: vec3<f32>,
    @location(1) tex: vec2<f32>,
}

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.pos = view_proj * vec4<f32>(in.pos, 1.0);
    out.nor = in.nor;
    out.tex = in.tex;
    return out;
}

@group(0) @binding(1) var texture_atlas: texture_2d<f32>;
@group(0) @binding(2) var sample_atlas: sampler;

@fragment
fn fs_main(out: VertexOut) -> @location(0) vec4<f32> {
    let color = textureSample(texture_atlas, sample_atlas, out.tex);

    if color.a < EPS {
        discard;
    }

    let abs_nor = abs(out.nor);
    let light = dot(abs_nor, vec3<f32>(1.0, 0.6, 0.4));

    return color * light;
}
