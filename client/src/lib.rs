#[macro_use]
extern crate log;

mod state;
mod types;
mod controller;
pub mod connection;

pub mod ggez_prelude {
    pub use ggez::{
        Context,
        ContextBuilder,
        GameResult,
        conf::{WindowSetup, WindowMode},
        event::{self, EventHandler, MouseButton},
        graphics::{
            self,
            set_screen_coordinates,
            Color,
            Mesh,
            MeshBuilder,
            Rect,
        },
        nalgebra as na,
    };
}

pub mod nalgebra_prelude {
    pub use ggez::nalgebra as na;
    pub type Point2 = na::Point2<f32>;
}

mod consts {
    pub const WINDOW_WIDTH: f32 = 1280.0; // TODO: Remove these. Retrieve dynamically.
    pub const WINDOW_HEIGHT: f32 = 720.0;
}

pub use state::State;