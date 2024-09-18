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

    pub fn mov(&mut self, movement: DegreesOfMovement, elapsed: f64) {
        self.translate(movement.transitional, elapsed);
    }
    fn translate(&mut self, movement: TranslationalEnvelops, elapsed: f64) {
        let dist = self.speed * elapsed;
        let surge = match movement.surge {
            None => 0.,
            Some(Surge::Forward) => -1.,
            Some(Surge::Backward) => 1.,
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
            let sway = {
                let direction = [
                    -self.facing.dims()[2].get(),
                    0.,
                    self.facing.dims()[0].get(),
                ];
                let mut direction = Vector::new(direction.map(|x| FiniteF64::new(x).unwrap()));
                direction.mul(sway);
                direction
            };
            let surge = {
                let direction = [self.facing.dims()[0].get(), 0., self.facing.dims()[2].get()];
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
        self.position = self.position.sub(&translation);
        dbg!(&self.position);
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
pub struct TranslationalEnvelops {
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
pub struct DegreesOfMovement {
    pub transitional: TranslationalEnvelops,
}
