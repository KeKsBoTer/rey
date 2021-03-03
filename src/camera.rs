use cgmath::{Point3, Vector2, Vector3};

use crate::ray::{self, Ray};

#[derive(Debug)]
pub struct Camera {
    // TODO add support for camera rotation
    pos: Point3<f64>,

    // distance from camera pos to view frame
    focal_length: f64,

    // size of view frame (x,y)
    s: Vector2<f64>,
}

impl Camera {
    pub fn new(sx: f64, sy: f64, focal_length: f64) -> Camera {
        Camera {
            pos: Point3::new(0., 0., 0.),
            focal_length: focal_length,
            s: Vector2::new(sx, sy),
        }
    }

    pub fn ray(&self, u: f64, v: f64) -> ray::Ray {
        Ray::new(
            self.pos,
            Vector3::new(
                (u - 0.5) * self.s.x,
                -(v - 0.5) * self.s.y,
                self.focal_length,
            )
        )
    }
}
