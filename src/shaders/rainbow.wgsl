struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) col: vec3<f32>,
    @location(2) tex: vec2<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) col: vec3<f32>,
    @location(1) tex: vec2<f32>,
};

@vertex 
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.pos = vec4<f32>(in.pos, 1.0);
    out.col = in.col;
    out.tex = in.tex;
    return out;
}

@fragment
fn fs_main(out: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(out.col, 1.0);
}
