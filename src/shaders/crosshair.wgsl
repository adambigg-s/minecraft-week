struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) col: vec3<f32>,
    @location(2) tex: vec2<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local_pos: vec3<f32>,
    @location(1) col: vec3<f32>,
    @location(2) tex: vec2<f32>,
};

@vertex 
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.pos = vec4<f32>(in.pos, 1.0);
    out.local_pos = out.pos.xyz;
    out.col = in.col;
    out.tex = in.tex;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let x = in.local_pos.x;
    let y = in.local_pos.y;

    if x < -0.003 || x > 0.003 {
        discard;
    }
    if y < -0.005 || y > 0.005 {
        discard;
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.7);
}
