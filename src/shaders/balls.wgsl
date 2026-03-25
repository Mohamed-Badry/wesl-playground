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
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );

    var uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(2.0, 0.0),
        vec2<f32>(0.0, 2.0)
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
    out.uv = uv[in_vertex_index];
    return out;
}

const PI = 3.14159;

fn bubble(st: vec2<f32>, pos: vec2<f32>, r: f32) -> f32 {
    return smoothstep(distance(st, pos), r + 0.003, r);
}

fn circle_around(time: f32, dist: vec2<f32>, shift: f32) -> vec2<f32> {
    return vec2f(dist.x * sin(2.0 * time + shift), dist.y * cos(2.0 * time + shift));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var st = in.uv;
    let aspect = u.resolution.x / u.resolution.y;
    st.x *= aspect;
    let center = vec2f(0.5 * aspect, 0.5);
    let mouse_centered = u.mouse - center;

    let radius = 0.12;
    let dist = mouse_centered;

    const n_circles = 20;
    var pct = 1.;
    pct = bubble(st, center, radius);

    for (var i = 0; i < n_circles; i++) {
        let shift = f32(i) / f32(n_circles) * 2.0 * PI;
        pct *= bubble(st, center + circle_around(u.time, dist, shift), radius);
    }

    for (var i = 0; i < n_circles; i++) {
        let shift = f32(i) / f32(n_circles) * 2.0 * PI;
        let perp = vec2(-dist.y, dist.x);
        pct *= bubble(st, center + circle_around(u.time, perp, shift), radius);
    }

    var color = vec3f(pct);
    // color.r += 0.4;
    color.g += 0.4;
    color.b += 0.4;

    return vec4f(fract(color * 5.113), 1.0);
}