@group(0)
@binding(0)
var<storage, read> in_buf: array<u32>;

@group(0)
@binding(1)
var<storage, read_write> out_buf: array<u32>;

@group(0)
@binding(2)
var<uniform> global_buf: u32;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let global = global_buf;
    out_buf[id.x] = in_buf[id.x];
}
