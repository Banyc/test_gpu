struct Uniform {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    padding: vec3<u32>,
    sin: f32,
}
@group(0)
@binding(0)
var<uniform> uniform: Uniform;
@group(0)
@binding(1)
var texture: texture_2d<f32>;
@group(0)
@binding(2)
var sampl: sampler;

struct Vertex {
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
}

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
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

    let pos = uniform.projection * uniform.view * uniform.model * vec4<f32>(vertex.position, 1.0);
    // let pos = vec4<f32>(vertex.position, 1.0);

    var fragment = Fragment();
    fragment.position = pos;
    fragment.color = vertex.color;
    fragment.tex_coord = vertex.tex_coord;
    return fragment;
}

@fragment
fn fs_main(fragment: Fragment) -> Color {
    // var color = Color();
    // color.color[1] = uniform.sin;
    // return color;

    var solid_color = vec4<f32>(fragment.color, 1.0);
    solid_color[1] = uniform.sin;
    let tex_color = textureSample(texture, sampl, fragment.tex_coord);
    var color = Color();
    color.color = solid_color * tex_color;
    return color;
}
