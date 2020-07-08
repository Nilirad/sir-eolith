use crate::nalgebra_prelude::*;
use derive_more::{Add, AddAssign};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Add, AddAssign)]
pub struct Milliseconds(pub u128);

pub type SnakePoint = Vec2;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PlayerType {
    You,
    Other,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PlayerTag(pub PlayerType);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FoodTag;

#[derive(Copy, Clone, PartialEq)]
pub struct Id(pub u16);

#[derive(Copy, Clone, PartialEq, AddAssign)]
pub struct Pos(pub Vec2);

impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

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
}