use crate::nalgebra_prelude::*;
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

#[derive(Copy, Clone, PartialEq)]
pub struct Id(pub u16);

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
}