struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
};

struct Uniform {
  model_matrix: mat4x4<f32>,
  view_matrix: mat4x4<f32>,
  projection_matrix: mat4x4<f32>,
  normal_matrix: mat3x3<f32>,
  resolution: vec2<f32>,
  elapsedTime: f32, // in seconds
};

@group(0) @binding(0)
var<uniform> unif: Uniform;

@vertex
fn vs_main(
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
) -> VertexOutput {
  var out: VertexOutput;
  out.position = unif.projection_matrix * unif.view_matrix * unif.model_matrix * vec4<f32>(position, 1.0);
  out.normal = normalize(unif.normal_matrix * normal);
  out.uv = uv;
  return out;
}

// ----------------------------------------------------
// Translated GLSL Functions and Fragment Shader Below
// ----------------------------------------------------

fn rand(r: f32) -> f32 {
    return fract(sin(r * 12.9898) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // in.position contains the fragment coordinates (equivalent to gl_FragCoord)
    var st: vec2<f32> = in.position.xy / unif.resolution.xy;
    st.x = st.x * (unif.resolution.x / unif.resolution.y);

    var color: vec3<f32> = vec3<f32>(0.0);

    const TOTAL_POINTS: i32 = 100;
    var point_arr: array<vec2<f32>, 100>;

    for (var i: i32 = 0; i < TOTAL_POINTS - 1; i++) {
        var r: vec2<f32> = vec2<f32>(f32(i) * 43893.32, f32(i) * 209320.0);
        point_arr[i] = vec2<f32>(
            rand(r.x) + 0.6 * sin(unif.elapsedTime * rand(r.y)),
            rand(r.y) + 0.6 * cos(unif.elapsedTime * rand(r.x))
        );
    }
    
    // Fallback for u_mouse since it isn't in the Uniform struct.
    // If you add `mouse: vec2<f32>` to the Uniform struct, change this to `unif.mouse`.
    let u_mouse = vec2<f32>(0.0, 0.0); 
    point_arr[TOTAL_POINTS - 1] = u_mouse / unif.resolution;

    var m_dist: f32 = 1.0;

    for (var i: i32 = 0; i < TOTAL_POINTS; i++) {
        var dist: f32 = distance(st, point_arr[i]);
        m_dist = min(m_dist, dist);
    }

    color = color + vec3<f32>(m_dist);

    return vec4<f32>(color, 1.0);
}
