use crate::nalgebra_prelude::*;
use derive_more::{Add, AddAssign};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Add, AddAssign)]
pub struct Milliseconds(pub u128);

#[derive(Copy, Clone)]
pub struct SnakePoint(pub Vec2);

impl SnakePoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

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

#[derive(Copy, Clone, PartialEq)]
pub struct Pos(pub Vec2);

impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

pub type Fullness = f32;

pub struct Length {
    pub points: Vec<SnakePoint>,
    pub fullness: Fullness,
}

impl Length {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            fullness: 0.0, // maybe add a parameter?
        }
    }

    pub fn push(&mut self, value: SnakePoint) {
        self.points.push(value);
    }

    pub fn remove_tail(&mut self) {
        self.points.remove(0); // TODO: Checking if exist? Use VecDeque?
    }

    pub fn iter(&self) -> std::slice::Iter<SnakePoint> {
        self.points.iter()
    }
}