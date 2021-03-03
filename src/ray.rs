use cgmath::{InnerSpace, Point3, Vector3};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    // ray origin point
    pub orig: Point3<f64>,
    // ray direction
    pub dir: Vector3<f64>
}

impl Ray {
    pub fn new(orig: Point3<f64>, dir: Vector3<f64>) -> Ray {
        Ray {
            orig: orig,
            dir: dir.normalize()
        }
    }

    pub fn pos(&self, lambda: f64) -> Point3<f64> {
        self.orig + lambda * self.dir
    }
}

pub struct Intersection{
    pub pos: Point3<f64>,
    pub normal: Vector3<f64>,
    pub lambda: f64,
    pub color: Vector3<f64>
}

impl Intersection {
    pub fn cast(&self, dir:Vector3<f64>) -> Ray {
        Ray{
            orig:self.pos,
            dir:dir
        }
    }

    pub fn diffuse(&self) -> Ray{
        let mut rng = rand::thread_rng();
        let mut nv: Vector3<f64>;
        loop {
            let n: [f64; 3] = rng.gen();
            nv = 2.*Vector3::new(n[0]-0.5, n[1]-0.5, n[2]-0.5);
            if nv.magnitude2() <= 0.5 {
                break;
            }
        }
        return self.cast((2.*nv+self.normal).normalize());
    }
}