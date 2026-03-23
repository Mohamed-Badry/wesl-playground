struct Uniforms {
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    time: f32,
}

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {

    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), // Bottom-left
        vec2<f32>(3.0, -1.0), // Far bottom-right 
        vec2<f32>(-1.0, 3.0)// Far top-left 
    );

    // Maps to 1x1 coordinate box, bottom-left origin
    var uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.0), // Bottom-left
        vec2<f32>(2.0, 0.0), // Maps right edge of screen to x=1.0
        vec2<f32>(0.0, 2.0)// Maps top edge of screen to y=1.0
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    out.uv = uv[vertex_index];
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = in.uv.x;
    let g = in.uv.y * u.time;
    let b = 0.5 + 0.5 * sin(u.time);
    return vec4<f32>(r, g, b, 1.0);
}
