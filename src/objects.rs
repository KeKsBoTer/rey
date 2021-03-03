use core::f64;

use crate::ray::{Ray,Intersection};

use cgmath::{InnerSpace, Point3, Vector3};

const EPS:f64 = 1e-8;

pub trait Intersect {
    // checks if object intersects with ray
    // returns reflected ray direction if it does
    fn intersects(&self, ray: Ray) -> Option<Intersection>;
}

pub struct IntersectList {
    objects: Vec<Box<dyn Intersect>>,
}

impl IntersectList {
    pub fn new(objects: Vec<Box<dyn Intersect>>) -> IntersectList {
        return IntersectList { objects: objects };
    }
}

impl Intersect for IntersectList {
    fn intersects(&self, ray: Ray) -> Option<Intersection> {
        // self.objects.iter()
        // .filter_map(|o|o.intersects(ray))
        // .min_by_key(|o|((o.orig-ray.orig).magnitude2()*1000.) as i32)
        let mut result : Option<Intersection> = None;
        for o in self.objects.iter(){
            if let Some(v) = o.intersects(ray){
                result = Some(match result {
                    Some(v2) => if v.lambda < v2.lambda {v} else {v2},
                    None => v,
                });
            }
        }
        result 
    }
}

#[derive(Debug)]
pub struct Sphere {
    pub radius: f64,
    pub center: Point3<f64>,
    pub color: Vector3<f64>,
}

impl Intersect for Sphere {
    fn intersects(&self, ray: Ray) -> Option<Intersection> {
        let d = ray.orig - self.center;
        let vd: f64 = ray.dir.dot(d).into();

        let dd: f64 = d.magnitude2().into();

        let r = self.radius;

        let s = vd * vd - dd + r * r;
        if s <= 0. {
            return None;
        }
        let ss = s.sqrt();
        let l1 = -vd + ss;
        let l2 = -vd - ss;

        let lambda: f64;

        if l2 > EPS {
            lambda = l2
        } else if l1 > EPS{
            lambda = l1
        } else {
            return None;
        }

        let intersect = ray.orig + lambda * ray.dir;

        let s_norm: Vector3<f64> = (intersect - self.center) / r;

        // do not use Ray::new to avoid dir normalization (it is allready normalized)
        return Some(Intersection {
            pos: intersect,
            normal: s_norm,
            color: self.color,
            lambda:lambda,
        });
    }
}

#[derive(Debug)]
pub struct Ground {
    y: f64,
    color: Vector3<f64>,
}

impl Ground {
    pub fn new(y: f64, color: Vector3<f64>) -> Ground {
        Ground { y: y, color: color }
    }
}

impl Intersect for Ground {
    fn intersects(&self, ray: Ray) -> Option<Intersection> {
        let l = (self.y - ray.orig.y) / ray.dir.y;
        if l < EPS {
            return None;
        }
        // do not use Ray::new to avoid dir normalization (it is allready normalized)
        return Some(Intersection {
            pos: ray.pos(l),
            normal: Vector3::new(0., 1., 0.),
            color: self.color,
            lambda:l,
        });
    }
}
