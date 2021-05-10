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

let num_triangle_quads:u32 = 4u;
let triangles:array<array<Triangle,4>,3> = array<array<Triangle,4>,3>(
    array<Triangle,4>(// Bottom
     Triangle(array<vec3<f32>,3>(vec3<f32>(-10.0,-1.0,-10.0),vec3<f32>(10.0,-1.0,10.0),vec3<f32>(10.0,-1.0,-10.0)),
              0u),
     Triangle(array<vec3<f32>,3>(vec3<f32>(-10.0,-1.0,-10.0),vec3<f32>(-10.0,-1.0,10.0),vec3<f32>(10.0,-1.0,10.0)),
              0u),
     // Left
     Triangle(array<vec3<f32>,3>(vec3<f32>(-10.0,-1.0,-10.0),vec3<f32>(-10.0,9.0,10.0),vec3<f32>(-10.0,-1.0,10.0)),
              1u),
     Triangle(array<vec3<f32>,3>(vec3<f32>(-10.0,-1.0,-10.0),vec3<f32>(-10.0,9.0,-10.0),vec3<f32>(-10.0,9.0,10.0)),
              1u)
    ),
    array<Triangle,4>(// Right
     Triangle(array<vec3<f32>,3>(vec3<f32>(10.0,-1.0,-10.0),vec3<f32>(10.0,-1.0,10.0),vec3<f32>(10.0,9.0,10.0)), 2u),
     Triangle(array<vec3<f32>,3>(vec3<f32>(10.0,-1.0,-10.0),vec3<f32>(10.0,9.0,10.0),vec3<f32>(10.0,9.0,-10.0)), 2u),
     // Back
     Triangle(array<vec3<f32>,3>(vec3<f32>(-10.0,-1.0,10.0),vec3<f32>(10.0,9.0,10.0),vec3<f32>(10.0,-1.0,10.0)), 0u),
     Triangle(array<vec3<f32>,3>(vec3<f32>(-10.0,-1.0,10.0),vec3<f32>(-10.0,9.0,10.0),vec3<f32>(10.0,9.0,10.0)),
              0u)
    ),
    array<Triangle,4>(// Top
     Triangle(array<vec3<f32>,3>(vec3<f32>(-10.0,9.0,-10.0),vec3<f32>(10.0,9.0,-10.0),vec3<f32>(10.0,9.0,10.0)), 0u),
     Triangle(array<vec3<f32>,3>(vec3<f32>(-10.0,9.0,-10.0),vec3<f32>(10.0,9.0,10.0),vec3<f32>(-10.0,9.0,10.0)), 0u),
     // Front
     Triangle( array<vec3<f32>,3>(vec3<f32>(-10.0,-1.0,-100.0),vec3<f32>(10.0,-1.0,-100.0),vec3<f32>(10.0,9.0,-100.0)),
         0u),
     Triangle(array<vec3<f32>,3>(vec3<f32>(-10.0,-1.0,-100.0),vec3<f32>(10.0,9.0,-100.0),vec3<f32>(-10.0,9.0,-100.0)),
         0u)
    )
);

let num_spheres:u32 = 3u; // TODO replace once arrayLength works
let spheres: array<Sphere,3> = array<Sphere,3>(
    Sphere(0.5, vec3<f32>(0.0, 7.0, 0.0), 4u),
    Sphere(2.0, vec3<f32>(1.0, 1.0, 0.0), 3u),
    Sphere(3.0, vec3<f32>(-4.0, 2.0, 0.0), 3u),
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

var intersec: Intersection = Intersection(
    vec3<f32>(0.0,0.0,0.0),
    vec3<f32>(0.0,0.0,0.0),
    Material(vec4<f32>(0.0,0.0,0.0,0.0)),
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
        intersec.material = materials[s.materialIdx];
        return true;
    }
    return false;
}

var hit_index:u32 = 0u;

fn intersect(ray:Ray, v01:vec3<f32>, v11:vec3<f32>, v21:vec3<f32>, v02:vec3<f32>, v12:vec3<f32>,
                v22:vec3<f32>, v03:vec3<f32>, v13:vec3<f32>, v23:vec3<f32>, v04:vec3<f32>, v14:vec3<f32>,
                v24:vec3<f32>) -> f32{

    let e11 = v11 - v01;
    let e21 = v21 - v01;
    let e12 = v12 - v02;
    let e22 = v22 - v02;
    let e13 = v13 - v03;
    let e23 = v23 - v03;
    let e14 = v14 - v04;
    let e24 = v24 - v04;
    let v0x = vec4<f32>(v01.x, v02.x, v03.x, v04.x);
    let v0y = vec4<f32>(v01.y, v02.y, v03.y, v04.y);
    let v0z = vec4<f32>(v01.z, v02.z, v03.z, v04.z);
    let e1x = vec4<f32>(e11.x, e12.x, e13.x, e14.x);
    let e1y = vec4<f32>(e11.y, e12.y, e13.y, e14.y);
    let e1z = vec4<f32>(e11.z, e12.z, e13.z, e14.z);
    let e2x = vec4<f32>(e21.x, e22.x, e23.x, e24.x);
    let e2y = vec4<f32>(e21.y, e22.y, e23.y, e24.y);
    let e2z = vec4<f32>(e21.z, e22.z, e23.z, e24.z);
    let dir4x = ray.dir.xxxx;
    let dir4y = ray.dir.yyyy;
    let dir4z = ray.dir.zzzz;
    let pvecx = dir4y * e2z - dir4z * e2y;
    let pvecy = dir4z * e2x - dir4x * e2z;
    let pvecz = dir4x * e2y - dir4y * e2x;
    let divisor = pvecx * e1x + pvecy * e1y + pvecz * e1z;
    let invDivisor = vec4<f32>(1.0, 1.0, 1.0, 1.0) / divisor;
    let orig4x = ray.orig.xxxx;
    let orig4y = ray.orig.yyyy;
    let orig4z = ray.orig.zzzz;
    let tvecx = orig4x - v0x;
    let tvecy = orig4y - v0y;
    let tvecz = orig4z - v0z;
    let u4 = (tvecx * pvecx + tvecy * pvecy + tvecz * pvecz) * invDivisor;
    let qvecx = tvecy * e1z - tvecz * e1y;
    let qvecy = tvecz * e1x - tvecx * e1z;
    let qvecz = tvecx * e1y - tvecy * e1x;
    let v4 = (dir4x * qvecx + dir4y * qvecy + dir4z * qvecz) * invDivisor;
    let t4 = (e2x * qvecx + e2y * qvecy + e2z * qvecz) * invDivisor;
    var t : f32 = 1.0 / 0.0;
    if (t4.x < t && t4.x > MIN_DISTANCE){
        if (u4.x >= 0.0 && v4.x >= 0.0 && u4.x + v4.x <= 1.0) {
            t = t4.x;
            hit_index = 0u;
        }
    }
    if (t4.y < t && t4.y > MIN_DISTANCE){
        if (u4.y >= 0.0 && v4.y >= 0.0 && u4.y + v4.y <= 1.0) {
            t = t4.y;
            hit_index = 1u;
        }
    }
    if (t4.z < t && t4.z > MIN_DISTANCE){
        if (u4.z >= 0.0 && v4.z >= 0.0 && u4.z + v4.z <= 1.0) {
            t = t4.z;
            hit_index = 2u;
        }
    }
    if (t4.w < t && t4.w > MIN_DISTANCE){
        if (u4.w >= 0.0 && v4.w >= 0.0 && u4.w + v4.w <= 1.0) {
            t = t4.w;
            hit_index = 3u;
        }
    }
    return t;
}


fn quad_triangle_intersection(ray:Ray, t: array<Triangle,4>) -> bool {
    let hit_index: i32 = -1;
    let lambda =
        intersect(ray, t[0].points[0], t[0].points[1], t[0].points[2],
                  t[1].points[0], t[1].points[1], t[1].points[2],
                  t[2].points[0], t[2].points[1], t[2].points[2],
                  t[3].points[0], t[3].points[1], t[3].points[2]);

    if (lambda < intersec.lambda) {
        let normal =
            normalize(cross(t[hit_index].points[1] - t[hit_index].points[0],
                            t[hit_index].points[2] - t[hit_index].points[1]));
        intersec.pos = ray.orig + ray.dir * lambda;
        intersec.normal = normal;
        intersec.material = materials[t[hit_index].materialIdx];

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

    for (var t:u32 = 0u; t < num_triangle_quads; t = t+1u) {
        anyHit = quad_triangle_intersection(ray, triangles[t]) || anyHit;
    }
    return anyHit;
}


fn jitter(d:vec3<f32>, phi:f32, sina:f32, cosa:f32) -> vec3<f32> {
    let w = d;
    let u = normalize(cross(w.yzx, w));
    let v = cross(w, u);
    return (u * cos(phi) + v * sin(phi)) * sina + w * cosa;
}


fn lightColor(inc_ray:Ray) -> vec3<f32> {

    var ray:Ray= inc_ray;

    var specularBounce : bool= true;

    let light = spheres[0];
    let lightMaterial = materials[light.materialIdx];

    var color : vec3<f32> = vec3<f32>(0.0,0.0,0.0);
    var mask : vec3<f32> = vec3<f32>(1.0,1.0,1.0);
    for (var hits:u32 = 0u; hits <= MAX_DEPTH; hits=hits+1u) {
        if (!hitScene(ray)){
            break;
        }

        if (intersec.material.color.a > 0.0) {
            // material is emmisivespecularBounce
            if (specularBounce){
                color = color + (mask * intersec.material.color.a);
            }
            return color;
        }

        specularBounce = false;

        var c:f32 = 1.0;
        if (dot(intersec.normal, ray.dir) >= 0.0 ){
            c = -1.0;
        }

        let r2 = random();
        let d = jitter(c * intersec.normal, 2. * PI * random(), sqrt(r2), sqrt(1. - r2));

        // calc incomming light
        mask = mask * intersec.material.color.rgb;
        color =  color + (mask * intersec.material.color.a);

        ray = Ray(intersec.pos, d);

        let lightRay = (light.center) - intersec.pos;
        let lightDir = normalize(lightRay);

        if (dot(-lightDir, c * intersec.normal) >= 0.0) {
            continue;
        }
        // TODO add max distance param to hitScene inorder to make light check
        // easier
        let _ = hitScene(Ray(intersec.pos, lightDir));
        let o_normal = intersec.normal;
        if (abs(intersec.lambda - length(lightRay) + light.radius) <= MIN_DISTANCE) {

            let cos_a_max =
                sqrt(1.0 - clamp(1.0 / (intersec.lambda * intersec.lambda), 0.0, 1.0));
            let weight = 2.0 * (1.0 - cos_a_max);
            // calc next event estimation
            color = color + (mask * lightMaterial.color.rgb * lightMaterial.color.a) *
                     (weight * clamp(dot(lightDir, o_normal), 0., 1.));
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

    let sun_dir = normalize(vec3<f32>(0.0, 0.0, -1.0));

    let rnd = randomUnitVector();

    let ray = cast_ray_from_camera(c, vec2<f32>(pix) / vec2<f32>(size) + rnd.xy / (2.0 * f32(size.x)));

    var colorOut: vec3<f32> = lightColor(ray);

    // mix with color of last frame
    let lastColor = textureLoad(framebuffer_src, vec2<i32>(pix)).rgb;
    let factor = 1.0 / f32(min(uniforms.pass + 1u, 200u));
    colorOut = mix(lastColor, colorOut, vec3<f32>(factor,factor,factor));

    textureStore(framebuffer_dst, vec2<i32>(pix), vec4<f32>(colorOut, 1.0));
}