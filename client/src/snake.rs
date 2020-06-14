use crate::nalgebra_prelude::*;

#[derive(Copy, Clone)]
pub struct SnakeSegment(pub Point2);

impl SnakeSegment {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Point2::new(x, y))
    }
}