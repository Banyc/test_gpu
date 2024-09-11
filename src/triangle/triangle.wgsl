struct Uniform {
    green: f32,
}
@group(0)
@binding(0)
var<uniform> uniform: Uniform;

struct Vertex {
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

struct Color {
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: Vertex) -> Fragment {
    // let x = f32(i32(vertex.index) - 1);
    // let y = f32(i32(vertex.index & 1u) * 2 - 1);
    // var fragment = Fragment();
    // fragment.position = vec4<f32>(x, y, 0.0, 1.0);

    var fragment = Fragment();
    fragment.position = vec4<f32>(vertex.position, 1.0);
    fragment.color = vertex.color;
    return fragment;
}

@fragment
fn fs_main(fragment: Fragment) -> Color {
    var color = Color();
    color.color = vec4<f32>(fragment.color, 1.0);
    color.color[1] = uniform.green;
    return color;
}
