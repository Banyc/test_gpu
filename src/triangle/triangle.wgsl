struct Uniform {
    green: f32,
}
@group(0)
@binding(0)
var<uniform> uniform: Uniform;

struct Vertex {
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
}

struct Fragment {
    @builtin(position) position: vec4<f32>,
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
    return fragment;
}

@fragment
fn fs_main(fragment: Fragment) -> Color {
    var color = Color();
    let green = uniform.green;
    color.color = vec4<f32>(1.0, green, 0.2, 1.0);
    return color;
}
