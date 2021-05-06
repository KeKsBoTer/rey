[[group(0), binding(0)]]
var framebuffer_src: [[access(read)]] texture_storage_2d<rgba16float>;

[[group(0), binding(1)]]
var framebuffer_dst: [[access(write)]] texture_storage_2d<rgba16float>;

[[stage(compute), workgroup_size(32, 16)]]
fn main([[builtin(global_invocation_id)]] gid: vec3<u32>) {
    let pix = vec2<i32>(i32(gid.x), i32(gid.y));
    let size = textureDimensions(framebuffer_dst);
    if (pix.x >= size.x || pix.y >= size.y) {
        return;
    }
	let color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    textureStore(framebuffer_dst, pix, color);
}