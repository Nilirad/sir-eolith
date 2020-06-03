mod state;
mod ws;
mod utils;

type Msg = Vec<u8>;

pub mod ggez_prelude {
    pub use ggez::{
        Context,
        ContextBuilder,
        GameResult,
        conf::{WindowSetup, WindowMode},
        event::{self, EventHandler},
        graphics::{
            self,
            set_screen_coordinates,
            Color,
            Mesh,
            Rect,
        },
        nalgebra::{self as na, Point},
    };
}

pub mod consts {
    use crate::ggez_prelude::*;

    pub const WINDOW_X: f32 = 1280.0;
    pub const WINDOW_Y: f32 = 720.0;
    pub const BACKGROUND_COLOR: Color = graphics::BLACK;
    pub const ORIGIN: &str = "http://slither.io";
    pub const SERVER_URL: &str = "ws://149.202.210.168:444/slither";
}

pub use state::State;