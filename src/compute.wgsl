let MIN_DISTANCE : f32 = 0.001;
let PI:f32 = 3.14159265359;
let MAX_DEPTH:u32 = 3u;

[[group(0), binding(0)]]
var framebuffer_src: [[access(read)]] texture_storage_2d<rgba16float>;

[[group(0), binding(1)]]
var framebuffer_dst: [[access(write)]] texture_storage_2d<rgba16float>;



[[block]]
struct Uniforms {
    u_view_proj: mat4x4<f32>;
    time: f32;
    pass: u32;
};

[[group(1), binding(0)]] var<uniform> uniforms: Uniforms;

struct Ray {
    orig: vec3<f32>;
    dir: vec3<f32>;
};

struct Material {
    color: vec4<f32>;
};

struct Intersection {
    pos: vec3<f32>;
    normal: vec3<f32>;
    material: Material;
    lambda: f32;
};

struct Camera {
    focal_length: f32;
    ratio: f32;
};

struct Sphere {
    radius: f32;
    center: vec3<f32>;
    materialIdx: u32;
};

struct Triangle {
    points: array<vec3<f32>,3>;
    materialIdx: u32;
};

let materials: array<Material,5> = array<Material, 5>(
    Material(vec4<f32>(0.7, 0.7, 0.7, 0.0)),
    Material(vec4<f32>(0.7, 0.0, 0.0, 0.0)),
    Material(vec4<f32>(0.0, 0.7, 0.0, 0.0)),
    Material(vec4<f32>(0.7, 0.7, 0.7, 0.0)), // sphere
    Material(vec4<f32>(1.0, 1.0, 1.0, 3.0)),       // light
);

var seed: u32 = 0u;

fn random() -> f32{
	seed = seed * 747796405u + 1u;
    var word: u32 = ((seed >> ((seed >> 28u) + 4u)) ^ seed) * 277803737u;
    word = (word >> 22u) ^ word;
    return f32(word) / 4294967295.0;
}

fn randomUnitVector() -> vec3<f32> {
    return normalize(vec3<f32>(random(), random(), random()) * 2.0 - 1.0);
}

fn cast_ray_from_camera(c: Camera, uv:vec2<f32>) -> Ray{
    let ray =
        normalize(vec3<f32>((uv.x - 0.5) * c.ratio, -(uv.y - 0.5), c.focal_length));
    let center = (uniforms.u_view_proj * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    return Ray(center,
               (uniforms.u_view_proj * vec4<f32>(ray, 1.0)).xyz - center);
}


fn sphere_intersection(s:Sphere, ray:Ray, i: ptr<private,Intersection>) -> bool{
    let d = ray.orig - s.center;
    let vd = dot(ray.dir, d);

    let dd = dot(d, d); // length squared

    let r = s.radius;

    let a = vd * vd - dd + r * r;
    if (a <= 0.) {
        return false;
    }
    let ss = sqrt(a);
    let l1 = f32(-vd + ss);
    let l2 = f32(-vd - ss);

	let int:Intersection = (*i);

    if (l2 > MIN_DISTANCE || l1 > MIN_DISTANCE) {
        let l = min(l1, l2);
        if (l > (int.lambda)) {
            return false;
		}

        // i.lambda = l;
        // let intersect = ray.orig + l * ray.dir;
        // let normal = (intersect - s.center) / s.radius;

        // i.pos = intersect;
        // i.normal = normal;
        // i.material = materials[s.materialIdx];
        return true;
    }
    return false;
}

[[stage(compute), workgroup_size(32, 16)]]
fn main([[builtin(global_invocation_id)]] gid: vec3<u32>) {
    let pix = gid.xy;
    let size = vec2<u32>(textureDimensions(framebuffer_dst));

    if (pix.x >= size.x || pix.y >= size.y) {
        return;
    }

    seed = (size.x * pix.y + pix.x) * (uniforms.pass + 1u);

	let rnd = randomUnitVector();

    let c = Camera(1.0, f32(size.x) / f32(size.y));
    let ray = cast_ray_from_camera(c, vec2<f32>(pix / size) + rnd.xy / (2.0 * f32(size.x)));

	let color = vec4<f32>(rnd, 1.0);
    textureStore(framebuffer_dst, vec2<i32>(pix), color);
}