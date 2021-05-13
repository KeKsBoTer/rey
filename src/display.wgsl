let positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
);

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(positions[in_vertex_index], 0.0, 1.0);
}


fn lessThan(f:vec3<f32>, value:f32) -> vec3<f32> {
    return vec3<f32>(
		select(1.0,0.0,f.x < value),
		select(1.0,0.0,f.y < value),
		select(1.0,0.0,f.z < value)
		);
}

// source https://www.shadertoy.com/view/tdXBW8
fn linearToSRGB(rgb:vec3<f32>) -> vec3<f32> {
    let rgb_c = clamp(rgb, vec3<f32>(0.0,0.0,0.0), vec3<f32>(1.0,1.0,1.0));

    return mix(pow(rgb_c * 1.055, vec3<f32>(1.0 / 2.4)) - 0.055, rgb_c * 12.92,
               lessThan(rgb_c, 0.0031308));
}

[[group(0), binding(0)]]
var texture: [[access(read)]] texture_storage_2d<rgba16float>;


[[stage(fragment)]]
fn fs_main([[builtin(position)]] position: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let size = textureDimensions(texture);
    let x = i32(position.x);
    let y = i32(position.y);
	let color = textureLoad(texture, vec2<i32>(x, y));
    return vec4<f32>(linearToSRGB(color.rgb),color.a);
}