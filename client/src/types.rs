use crate::nalgebra_prelude::*;
use derive_more::{Add, AddAssign};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Add, AddAssign)]
pub struct Milliseconds(pub u128);

#[derive(Copy, Clone)]
pub struct SnakeSegment(pub Point2);

impl SnakeSegment {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Point2::new(x, y))
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
pub struct Pos(pub Point2);

impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Point2::new(x, y))
    }
}

pub struct SnakeSegments(pub Vec<SnakeSegment>);

impl SnakeSegments {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, value: SnakeSegment) {
        self.0.push(value);
    }

    pub fn remove_tail(&mut self) {
        self.0.remove(0); // TODO: Checking if exist? Use VecDeque?
    }

    pub fn iter(&self) -> std::slice::Iter<SnakeSegment> {
        self.0.iter()
    }
}