const EPS: f32 = 1e-3;
const PI: f32 = acos(-1.0);

const AMBIENT: f32 = 0.125;
const FREQUENCY: f32 = 35.0;

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
    @location(0) nor: vec3<f32>,
    @location(1) tex: vec2<f32>,
    @location(3) fil: f32,
    @location(4) ao: f32,
}

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;

@vertex 
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.pos = view_proj * vec4<f32>(in.pos, 1.0);
    out.nor = in.nor;
    out.tex = in.tex;
    out.fil = in.fil;
    out.ao = in.ao;
    return out;
}

@group(0) @binding(2) var texture_atlas: texture_2d<f32>;
@group(0) @binding(3) var sample_atlas: sampler;
@group(0) @binding(4) var<uniform> global_time: f32;
@group(0) @binding(5) var<uniform> global_ao: f32;

@group(1) @binding(0) var<uniform> gen_time: f32;

fn rainbow(time: f32) -> vec3<f32> {
    let a = vec3<f32>(0.5);
    let b = vec3<f32>(0.5);
    let c = vec3<f32>(1.0);
    let d = vec3<f32>(0.0, 0.33, 0.67);
    return a + b * cos(2.0 * PI * (c * time + d));
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let color = textureSample(texture_atlas, sample_atlas, in.tex);
    if color.a < EPS {
        discard;
    }

    let ao = pow(in.ao, global_ao);
    let lum = clamp(in.fil, AMBIENT, 1.0);
    let final_color = color * ao * lum;

    return mix(final_color, vec4<f32>(rainbow(gen_time / FREQUENCY), 1.0), 0.3);
}


