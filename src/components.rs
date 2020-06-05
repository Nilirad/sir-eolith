use crate::snake::SnakeSegment;
use crate::nalgebra_prelude::*;

#[derive(Copy, Clone, PartialEq)]
pub enum Tags {
    You,
    OtherPlayer,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Id(pub u16);

#[derive(Copy, Clone)]
pub struct Pos(pub Point2);

impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Point2::new(x, y))
    }
}

pub struct SnakeSegments(Vec<SnakeSegment>);

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
}