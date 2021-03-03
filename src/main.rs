mod camera;
mod objects;
mod ray;
use crate::objects::Intersect;
use cgmath::{InnerSpace, Point3, Vector3};
use image::{Rgb, RgbImage};
use rand::Rng;
fn main() {
    let sp = objects::Sphere {
        radius: 2.,
        center: Point3::new(0., 1.0, 10.),
        color: Vector3::new(1., 0., 0.),
    };

    let g = objects::Ground::new(-1., Vector3::new(1., 0., 1.));

    let objs = objects::IntersectList::new(vec![Box::new(sp), Box::new(g)]);

    let sun = Vector3::new(0.5, 1., 0.).normalize();

    let c = camera::Camera::new(1., 1., 1.);

    let width = 320;
    let height = 320;

    let mut img = RgbImage::new(width, height);

    let ns = 20;

    let mut rng = rand::thread_rng();

    for i in 0..height {
        for j in 0..width {
            let mut fc = Vector3::new(0., 0., 0f64);
            for _s in 0..ns {
                let u = (j as f64 + rng.gen::<f64>()) / width as f64;
                let v = (i as f64 + rng.gen::<f64>()) / height as f64;

                let r = c.ray(u, v);

                let intersection = objs.intersects(r);
                let c = match intersection {
                    Some(v) => {
                        let mut c:Vector3<f64> = v.color;
                        let mut intersec = v;
                        for _b in 0..5{
                            let n = intersec.diffuse();
                            let ray_to_sun = objs.intersects(n);
                            if let Some(vn) = ray_to_sun{
                                intersec = vn; 
                                c *=0.5;
                            }else{
                                break;
                            }
                        }
                        c
                    }
                    None => Vector3::new(0., 0., 0.),
                };
                fc += c / ns as f64;
            }
            img.put_pixel(
                j,
                i,
                Rgb([
                    (255.0 * (fc[0]).sqrt()) as u8,
                    (255.0 * (fc[1]).sqrt()) as u8,
                    (255.0 * (fc[2]).sqrt()) as u8,
                ]),
            );
        }
    }
    img.save("render.png");
}
