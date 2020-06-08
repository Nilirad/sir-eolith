#[macro_use]
extern crate log;

mod state;
mod utils;
mod message;
mod components;
mod snake;
mod playback;
pub mod ws;

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

pub mod consts {
    use crate::ggez_prelude::*;

    pub const WINDOW_WIDTH: f32 = 1280.0;
    pub const WINDOW_HEIGHT: f32 = 720.0;
    pub const BACKGROUND_COLOR: Color = graphics::BLACK;
    pub const ORIGIN: &str = "http://slither.io";
    pub const SERVER_URL: &str = "ws://149.202.210.168:444/slither";
}

pub use state::State;