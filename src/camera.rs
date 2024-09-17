use math::vector::Vector;
use strict_num::FiniteF64;

use crate::transform::{look_at, TransformMatrix};

#[derive(Debug, Clone)]
pub struct Camera {
    speed: f64,
    position: Vector<3>,
    facing: Vector<3>,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            speed: 2.5,
            position: Vector::new([0., 0., 0.].map(|x| FiniteF64::new(x).unwrap())),
            facing: Vector::new([0., 0., -1.].map(|x| FiniteF64::new(x).unwrap())),
        }
    }
    pub fn set_speed(&mut self, v: f64) {
        self.speed = v;
    }
    pub fn set_position(&mut self, v: Vector<3>) {
        self.position = v;
    }
    pub fn set_facing(&mut self, mut v: Vector<3>) {
        v.normalize();
        self.facing = v;
    }

    pub fn mov(&mut self, direction: CameraMovement, elapsed: f64) {
        let dist = self.speed * elapsed;
        let dist = match direction {
            CameraMovement::Down | CameraMovement::Right | CameraMovement::Front => dist,
            CameraMovement::Up | CameraMovement::Left | CameraMovement::Back => -dist,
        };
        let dist = match direction {
            CameraMovement::Up | CameraMovement::Down => {
                Vector::new([0., dist, 0.].map(|x| FiniteF64::new(x).unwrap()))
            }
            CameraMovement::Left | CameraMovement::Right => {
                let direction = [
                    self.facing.dims()[2].get(),
                    0.,
                    -self.facing.dims()[0].get(),
                ];
                let mut direction = Vector::new(direction.map(|x| FiniteF64::new(x).unwrap()));
                direction.normalize();
                direction.mul(dist);
                direction
            }
            CameraMovement::Front | CameraMovement::Back => {
                let direction = [self.facing.dims()[0].get(), 0., self.facing.dims()[2].get()];
                let mut direction = Vector::new(direction.map(|x| FiniteF64::new(x).unwrap()));
                direction.normalize();
                direction.mul(dist);
                direction
            }
        };
        self.position = self.position.add(&dist);
    }

    pub fn view_matrix(&self) -> TransformMatrix {
        let at = self.position.add(&self.facing);
        look_at(
            self.position.dims().map(|x| x.get()),
            at.dims().map(|x| x.get()),
            [0., 1., 0.],
        )
    }
}
impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CameraMovement {
    Up,
    Down, //
    Left,
    Right, //
    Front,
    Back, //
}
