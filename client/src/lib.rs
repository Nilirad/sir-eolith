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
        event::{self, EventHandler, MouseButton, KeyCode, KeyMods},
        graphics::{
            self,
            set_screen_coordinates,
            Color,
            Mesh,
            MeshBuilder,
            Rect,
            Text,
            TextFragment,
            Scale,
        },
        nalgebra as na,
    };
}

pub mod nalgebra_prelude {
    pub use ggez::nalgebra as na;
    pub type Vec2 = na::Vector2<f32>;
}

mod consts {
    pub const WINDOW_WIDTH: f32 = 1280.0; // TODO: Remove these. Retrieve dynamically.
    pub const WINDOW_HEIGHT: f32 = 720.0;
    pub const MINIMAP_SIDE: usize = 80;
    pub const MINIMAP_AREA: usize = MINIMAP_SIDE * MINIMAP_SIDE;
}

pub use state::State;

const REV_ANGLE: f32 = 2.0 * std::f32::consts::PI;

fn modulo(n: f32, modulus: f32) -> f32 {
    let result = n % modulus;
    if result < 0.0 {
        result + modulus
    } else {
        result
    }
}