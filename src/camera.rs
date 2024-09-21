use std::f64::consts::PI;

use math::vector::Vector;
use strict_num::FiniteF64;

use crate::transform::{look_at, TransformMatrix};

const TRI_PERIOD: f64 = 2. * PI;

#[derive(Debug, Clone)]
pub struct Camera {
    speed: f64,
    position: Vector<3>,
    sensitivity: f64,
    yaw: f64,
    pitch: f64,
    fov: f64,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            speed: 2.5,
            position: Vector::new([0., 0., 0.].map(|x| FiniteF64::new(x).unwrap())),
            sensitivity: 0.1,
            pitch: 0.,
            yaw: -PI / 2.,
            fov: PI / 4.,
        }
    }
    pub fn set_speed(&mut self, v: f64) {
        self.speed = v;
    }
    pub fn set_position(&mut self, v: Vector<3>) {
        self.position = v;
    }
    pub fn position(&self) -> Vector<3> {
        self.position
    }
    pub fn set_yaw(&mut self, yaw: f64) {
        self.yaw = yaw % TRI_PERIOD;
    }
    pub fn set_pitch(&mut self, pitch: f64) {
        let near_perpendicular = PI / 2. - 0.001;
        self.pitch = (pitch % TRI_PERIOD).clamp(-near_perpendicular, near_perpendicular);
    }
    pub fn facing(&self) -> Vector<3> {
        let dims = [
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ]
        .map(|x| FiniteF64::new(x).unwrap());
        Vector::new(dims)
    }

    pub fn zoom(&mut self, offset: f64) {
        self.fov = (self.fov - offset).clamp(0.001, PI / 4.);
    }
    pub fn fov(&self) -> f64 {
        self.fov
    }

    pub fn rotate(&mut self, movement: RotationalMovement) {
        self.set_pitch(self.pitch + movement.pitch * self.sensitivity);
        self.set_yaw(self.yaw + movement.yaw * self.sensitivity);
        dbg!(self.yaw);
        dbg!(self.pitch);
        dbg!(&self.facing());
    }
    pub fn translate(&mut self, movement: TranslationalMovement, elapsed: f64) {
        let dist = self.speed * elapsed;
        let surge = match movement.surge {
            None => 0.,
            Some(Surge::Forward) => 1.,
            Some(Surge::Backward) => -1.,
        };
        let sway = match movement.sway {
            None => 0.,
            Some(Sway::Left) => -1.,
            Some(Sway::Right) => 1.,
        };
        let heave = match movement.heave {
            None => 0.,
            Some(Heave::Down) => -1.,
            Some(Heave::Up) => 1.,
        };
        let horizontal = || {
            if movement.sway.is_none() && movement.surge.is_none() {
                return None;
            }
            let facing = self.facing();
            let sway = {
                let direction = [-facing.dims()[2].get(), 0., facing.dims()[0].get()];
                let mut direction = Vector::new(direction.map(|x| FiniteF64::new(x).unwrap()));
                direction.mul(sway);
                direction
            };
            let surge = {
                let direction = [facing.dims()[0].get(), 0., facing.dims()[2].get()];
                let mut direction = Vector::new(direction.map(|x| FiniteF64::new(x).unwrap()));
                direction.mul(surge);
                direction
            };
            let mut horizontal = sway.add(&surge);
            horizontal.set_mag(dist);
            Some(horizontal)
        };
        let horizontal = horizontal();

        let vertical = || {
            movement.heave?;
            let mut vertical = Vector::new([0., heave, 0.].map(|x| FiniteF64::new(x).unwrap()));
            vertical.set_mag(dist);
            Some(vertical)
        };
        let vertical = vertical();

        let translation = match (horizontal, vertical) {
            (None, None) => return,
            (None, Some(x)) => x,
            (Some(x), None) => x,
            (Some(a), Some(b)) => a.add(&b),
        };
        self.position = self.position.add(&translation);
        dbg!(&self.position);
    }

    pub fn view_matrix(&self) -> TransformMatrix {
        let at = self.position.add(&self.facing());
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
pub struct TranslationalMovement {
    pub surge: Option<Surge>,
    pub sway: Option<Sway>,
    pub heave: Option<Heave>,
}

#[derive(Debug, Clone, Copy)]
pub enum Surge {
    Forward,
    Backward,
}
#[derive(Debug, Clone, Copy)]
pub enum Sway {
    Left,
    Right,
}
#[derive(Debug, Clone, Copy)]
pub enum Heave {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub struct RotationalMovement {
    pub yaw: f64,
    pub pitch: f64,
}
