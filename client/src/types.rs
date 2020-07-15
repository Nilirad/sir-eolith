use crate::nalgebra_prelude::*;
use crate::modulo;
use derive_more::{Add, AddAssign};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Add, AddAssign)]
pub struct Milliseconds(pub u128);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PlayerType {
    You,
    Other,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PlayerTag(pub PlayerType);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FoodTag;

pub type Id = u16;

pub type Pos = Vec2;

pub type SnakePoint = Pos;

pub type Size = f32;

pub type Fullness = f32;

pub struct Body {
    pub points: Vec<SnakePoint>,
    pub fullness: Fullness,
}

impl Body {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            fullness: 0.0, // maybe add a parameter?
        }
    }

    pub fn add_point(&mut self, value: SnakePoint) {
        self.points.push(value);
    }

    pub fn shrink(&mut self) {
        self.points.remove(0); // TODO: Checking if exist? Use VecDeque?
    }

    pub fn sc(&self) -> f32 {
        6f32.min(1.0 + (self.points.len() as f32 - 2.0) / 106.0)
    }

    pub fn scang(&self) -> f32 {
        0.13 + 0.87 * ((7.0 - self.sc()) / 6.0).powf(2.0)
    }
}

pub struct Movement {
    pub speed: f32,
    pub angle: f32,
    pub target_angle: f32,
    pub direction: f32,
}

impl Movement {
    const REV_ANGLE: f32 = 2.0 * std::f32::consts::PI;

    pub fn new(speed: f32, angle: f32, direction: f32) -> Self {
        Self {
            speed,
            angle,
            target_angle: angle,
            direction,
        }
    }

    pub fn spang(&self, spangdv: f32) -> f32 {
        (self.speed / spangdv).min(1.0)
    }

    pub fn update_angle(&mut self, mamu: f32, vfr: f32, scang: f32, spangdv: f32) {
        let delta_angle = mamu * vfr * scang * self.spang(spangdv);
        if self.direction != 0.0 {
            self.angle = modulo(self.angle - delta_angle, Self::REV_ANGLE);
            if self.ending_rotation() {
                self.angle = self.target_angle;
                self.direction = 0.0;
            }
        } else {
            self.angle = self.target_angle;
        }
    }

    fn ending_rotation(&self) -> bool {
        let mut h = (self.target_angle - self.angle) % Self::REV_ANGLE;
        if h < 0.0 {
            h += Self::REV_ANGLE;
            if h > std::f32::consts::PI {
                h -= Self::REV_ANGLE;
            }
        }

        h > 0.0
    }
}