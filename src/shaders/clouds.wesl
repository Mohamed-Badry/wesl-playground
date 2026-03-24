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

fn rand(r: f32) -> f32 {
    return fract(sin(r * 12.988) * 47.543);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var st = in.uv;
    let aspect = u.resolution.x / u.resolution.y;
    st.x = st.x * aspect;

    var color = vec3f(.0);

    const TOTAL_POINTS: i32 = 180;
    var point_arr: array<vec2<f32>, 180>;

    for (var i: i32 = 0; i < TOTAL_POINTS - 1; i++) {
        let r = vec2<f32>(f32(i) * .2332, f32(i) * .2020);

        // Multiply point X by aspect ratio to spread them across the whole window
        point_arr[i] = vec2<f32>(
            (rand(r.x) + 0.6 * sin(u.time * rand(r.y))) * aspect,
            rand(r.y) + 0.6 * cos(u.time * rand(r.x))
        );
    }

    point_arr[TOTAL_POINTS - 1] = u.mouse;

    var m_dist: f32 = .5;

    for (var i: i32 = 0; i < TOTAL_POINTS; i++) {
        let dist = distance(st, point_arr[i]);
        m_dist = min(m_dist, dist);
    }

    color += m_dist;
    // color = smoothstep(color, vec3f(0.05), vec3f(0.08 * sin(u.time * 5.0)));
    // color -= vec3f(.01);

    return vec4<f32>(0.7 - color + vec3f(0.0, 0.0, 0.5), 1.0);
}
