const EPS: f32 = 1e-3;

const FOG_START: f32 = 200.0;
const FOG_END: f32 = 550.0;
const FOG_EXP: f32 = 6.0;

const FACE_LIGHTING: vec3<f32> = vec3<f32>(0.5, 1.0, 0.3);
const FADE_COLOR: vec4<f32> = vec4<f32>(0.7, 0.8, 1.0, 1.0);

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex: vec2<f32>,
    @location(3) lum: f32,
    @location(4) ao: f32,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world_pos: vec4<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex: vec2<f32>,
    @location(3) lum: f32,
    @location(4) ao: f32,
}

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> view: mat4x4<f32>;

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.pos = view_proj * vec4<f32>(in.pos, 1.0);
    out.world_pos = view * vec4<f32>(in.pos, 1.0);
    out.nor = in.nor;
    out.tex = in.tex;
    out.lum = in.lum;
    out.ao = in.ao;
    return out;
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(2) var texture_atlas: texture_2d<f32>;
@group(0) @binding(3) var sample_atlas: sampler;
@group(0) @binding(4) var<uniform> global_time: f32;
@group(0) @binding(5) var<uniform> global_ao: f32;

@fragment
fn fs_main(in: VertexOut) -> FragmentOutput {
    let color = textureSample(texture_atlas, sample_atlas, in.tex);

    if color.a < EPS {
        discard;
    }

    let abs_nor = abs(in.nor);
    let light = dot(abs_nor, FACE_LIGHTING);
    let ao = pow(in.ao, global_ao);
    let diffuse_color = color * light * ao;

    let depth = length(in.world_pos.xyz);
    let fog_factor = pow(clamp((depth - FOG_START) / (FOG_END - FOG_START), 0.0, 1.0), FOG_EXP);

    let final_color = mix(diffuse_color, FADE_COLOR, fog_factor);

    var output: FragmentOutput;
    output.color = vec4<f32>(final_color.rgb * color.a, color.a);
    output.depth = in.pos.z;

    return output;
}
