let MIN_DISTANCE : f32 = 0.001;
let PI:f32 = 3.14159265359;
let MAX_DEPTH:u32 = 3u;

let SHOW_NORMALS: bool = false;

[[group(0), binding(0)]]
var framebuffer_src: [[access(read)]] texture_storage_2d<rgba16float>;

[[group(0), binding(1)]]
var framebuffer_dst: [[access(write)]] texture_storage_2d<rgba16float>;


[[block]]
struct Uniforms {
    u_view_proj: mat4x4<f32>;
    time: f32;
    pass: u32;
    num_samples: u32;
	num_faces: u32; // TODO remove when array
};

[[group(1), binding(0)]]
var<uniform> uniforms: Uniforms;

struct Vertex{
	x:f32;
	y:f32;
	z:f32;
};

[[block]]
struct Vertices{
	data:[[stride(12)]] array<Vertex>;
};

[[group(2), binding(0)]]
var<storage> vertices: [[access(read)]] Vertices;


[[block]]
struct Faces {
    data: [[stride(12)]] array<array<u32, 3>>;
};

[[group(2), binding(1)]]
var<storage> faces: [[access(read)]] Faces;

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
    materialIdx: u32;
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
	p1: vec3<f32>;
	p2: vec3<f32>;
	p3: vec3<f32>;
    materialIdx: u32;
};

let materials: array<Material,5> = array<Material, 5>(
    Material(vec4<f32>(0.7, 0.7, 0.7, 0.0)),
    Material(vec4<f32>(0.7, 0.0, 0.0, 0.0)),
    Material(vec4<f32>(0.0, 0.7, 0.0, 0.0)),
    Material(vec4<f32>(0.7, 0.7, 0.7, 0.0)), // sphere
    Material(vec4<f32>(1.0, 1.0, 1.0, 3.0)), // light
);


let num_spheres:u32 = 1u; // TODO replace once arrayLength works
let spheres: array<Sphere,1> = array<Sphere,1>(
    Sphere(10.0, vec3<f32>(250.0, 500.0, 100.0), 4u),
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
    let ray = vec3<f32>((uv.x - 0.5) * c.ratio, -(uv.y - 0.5), c.focal_length);
    let center = (uniforms.u_view_proj * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
	let dir = normalize((uniforms.u_view_proj * vec4<f32>(ray, 1.0)).xyz - center);
    return Ray(center,dir);
}

var intersec: Intersection = Intersection(
    vec3<f32>(0.0,0.0,0.0),
    vec3<f32>(0.0,0.0,0.0),
    0u,
    0.0,
);


fn sphere_intersection(s:Sphere, ray:Ray) -> bool{
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

    if (l2 > MIN_DISTANCE || l1 > MIN_DISTANCE) {
        let l = min(l1, l2);
        if (l > intersec.lambda) {
            return false;
        }

        intersec.lambda = l;
        let intersect = ray.orig + l * ray.dir;
        let normal = (intersect - s.center) / s.radius;

        intersec.pos = intersect;
        intersec.normal = normal;
        intersec.materialIdx = s.materialIdx;
        return true;
    }
    return false;
}

var hit_index:u32 = 0u;

fn triangle_intersection(ray:Ray, t: Triangle) -> bool {
    let edge1 = t.p2 - t.p1;
    let edge2 = t.p3 - t.p1;
    let h = cross(ray.dir, edge2);
    let a = dot(edge1, h);
    if (a > -MIN_DISTANCE && a < MIN_DISTANCE) {
        return false;
    }
    let f = 1.0 / a;
    let s = ray.orig - t.p1;
    let u = f * dot(s, h);
    if (u < 0.0 || u > 1.0) {
        return false;
    }
    let q = cross(s, edge1);
    let v = f * dot(ray.dir, q);
    if (v < 0.0 || u + v > 1.0) {
        return false;
    }
    let lambda = f * dot(edge2, q);

    if (lambda > MIN_DISTANCE && lambda < intersec.lambda) {
        let normal =
            normalize(cross(t.p1 - t.p2,
                            t.p3 - t.p1));

        intersec.pos = ray.orig + ray.dir * lambda;
        intersec.normal = normal;
        intersec.materialIdx = t.materialIdx;

        intersec.lambda = lambda;
        return true;
    }

    return false;
}


fn hitScene(ray:Ray) -> bool {
    var anyHit: bool = false;

    intersec.lambda = 1.0 / 0.0; // aka. infinity
    for (var s:u32 = 0u; s < num_spheres; s = s+1u){
        anyHit = sphere_intersection(spheres[s], ray) || anyHit;
    }

    for (var i:u32 = 0u; i < uniforms.num_faces; i = i+1u) {
		let face = faces.data[i];
		let p1 = face[0];
		let p2 = face[1];
		let p3 = face[2];

		let v1 = vertices.data[p1];
		let v2 = vertices.data[p3];
		let v3 = vertices.data[p2];


		let tri = Triangle(
			vec3<f32>(v1.x,v1.y,v1.z),
			vec3<f32>(v2.x,v2.y,v2.z),
			vec3<f32>(v3.x,v3.y,v3.z),
			0u
		);
        anyHit = triangle_intersection(ray, tri) || anyHit;
    }
    return anyHit;
}


fn jitter(d:vec3<f32>, phi:f32, sina:f32, cosa:f32) -> vec3<f32> {
    let w = d;
    let u = normalize(cross(w.yzx, w));
    let v = cross(w, u);
    return (u * cos(phi) + v * sin(phi)) * sina + w * cosa;
}


fn lightColor(init_ray:Ray) -> vec3<f32> {

    var ray:Ray = init_ray;

    var specularBounce : bool = true;

    let light = spheres[0];
    let lightMaterial = materials[light.materialIdx];

    var color : vec3<f32> = vec3<f32>(0.0,0.0,0.0);
    var mask : vec3<f32> = vec3<f32>(1.0,1.0,1.0);
    for (var hits:u32 = 0u; hits <= MAX_DEPTH; hits=hits+1u) {
        if (!hitScene(ray)){
            // TODO env map
            break;
        }
        
        let material = materials[intersec.materialIdx];

		if(SHOW_NORMALS){
			return (intersec.normal+1.0)/2.0 * (dot(intersec.normal,-ray.dir)+1.0)/2.0;
		}

        let emissiveness = material.color.a;

        if (emissiveness > 0.0) {
            // material is emmisivespecularBounce
            if (specularBounce){
                color = color + (mask * emissiveness );
            }
            return color;
        }

        specularBounce = false;

        let r2 = random();
        let d = jitter(intersec.normal, 2. * PI * random(), sqrt(r2), sqrt(1. - r2));


        // calc incomming light
        mask = mask * material.color.rgb;
        color =  color + (mask * emissiveness);


        ray.orig = intersec.pos;
        ray.dir =  d;

        let lightRay = (light.center + randomUnitVector() * light.radius) - intersec.pos;
        let lightDir = normalize(lightRay);

        // if (dot(-lightDir, c * intersec.normal) >= 0.0) {
        //     continue;
        // }

        let o_normal = intersec.normal;
        let hitAny = hitScene(Ray(intersec.pos, lightDir));

		// check if emissive stuff was hit
        if (hitAny && materials[intersec.materialIdx].color.a > 0.0){
			let sphere_radius = spheres[0].radius;

            let cos_a_max =
                sqrt(1.0 - clamp(sphere_radius * sphere_radius / (intersec.lambda * intersec.lambda), 0.0, 1.0));
            let weight = 2.0 * (1.0 - cos_a_max);
            // calc next event estimation
            color = color + (mask * lightMaterial.color.rgb * lightMaterial.color.a) *
                     (weight * dot(lightDir, o_normal));
        }
    }
    return color;
}

[[stage(compute), workgroup_size(32, 16)]]
fn main([[builtin(global_invocation_id)]] gid: vec3<u32>) {
    let pix = gid.xy;
    let size = vec2<u32>(textureDimensions(framebuffer_dst));

    if (pix.x >= size.x || pix.y >= size.y) {
        return;
    }

    seed = (size.x * pix.y + pix.x) * (uniforms.pass + 1u);

    let c = Camera(1.0, f32(size.x) / f32(size.y));

    let rnd = randomUnitVector();


    var colorOut: vec3<f32> = vec3<f32>(0.,0.,0.);
	for (var s:u32=0u;s<uniforms.num_samples; s = s+1u){
    	let ray = cast_ray_from_camera(c, vec2<f32>(pix) / vec2<f32>(size) + rnd.xy / (2.0 * f32(size.x)));
		colorOut = colorOut + lightColor(ray);
	}
	colorOut = colorOut / f32(uniforms.num_samples);

    // mix with color of last frame
    let lastColor = textureLoad(framebuffer_src, vec2<i32>(pix)).rgb;
    let factor = 1.0 / f32(min(uniforms.pass + 1u, 200u));
    colorOut = mix(lastColor, colorOut, vec3<f32>(factor,factor,factor));

    textureStore(framebuffer_dst, vec2<i32>(pix), vec4<f32>(colorOut, 1.0));
}