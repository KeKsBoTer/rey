use cgmath::{Point3, Vector2, Vector3};

use crate::ray::{self, Ray};

#[derive(Debug)]
pub struct Camera {
    // TODO add support for camera rotation
    pos: Point3<f32>,

    // distance from camera pos to view frame
    focal_length: f32,

    // size of view frame (x,y)
    s: Vector2<f32>,
}

impl Camera {
    pub fn new(sx: f32, sy: f32, focal_length: f32) -> Camera {
        Camera {
            pos: Point3::new(0., 0., 0.),
            focal_length: focal_length,
            s: Vector2::new(sx, sy),
        }
    }
}
