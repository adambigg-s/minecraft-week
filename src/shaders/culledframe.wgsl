const LINE_THICKNESS: f32 = 0.05;

struct VertexIn {
    @builtin(vertex_index) index: u32,
    @location(0) pos: vec3<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex: vec2<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) nor: vec3<f32>,
    @location(1) tex: vec2<f32>,
    @location(2) bar: vec3<f32>,
}

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;

@vertex 
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    out.pos = view_proj * vec4<f32>(in.pos, 1.0);
    out.nor = in.nor;
    out.tex = in.tex;
    switch in.index % 3 {
        case 0: {
            out.bar = vec3<f32>(1.0, 0.0, 0.0);
        }
        case 1: {
            out.bar = vec3<f32>(0.0, 1.0, 0.0);
        }
        case 2: {
            out.bar = vec3<f32>(0.0, 0.0, 1.0);
        }
        default: {}
    }
    return out;
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color: vec4<f32>};

@group(0) @binding(2) var texture_atlas: texture_2d<f32>;
@group(0) @binding(3) var sample_atlas: sampler;

@fragment
fn fs_main(in: VertexOut) -> FragmentOutput {
    var output: FragmentOutput;
    output.depth = in.pos.z;
    output.color = vec4<f32>(0.0);

    if in.bar.x < LINE_THICKNESS || in.bar.y < LINE_THICKNESS || in.bar.z < LINE_THICKNESS {
        output.color = vec4<f32>(abs(in.nor + vec3<f32>(0.33)), 1.0);
    }

    return output;
}

