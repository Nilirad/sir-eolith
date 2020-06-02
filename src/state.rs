use crate::ggez_prelude::*;
use crate::ws::WebSocket;
use crate::consts::*;

pub struct State {
    ws: WebSocket,
}

impl State {
    pub fn new() -> Self {
        Self {
            ws: WebSocket::new(SERVER_URL.to_owned()),
        }
    }
}

impl EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
        graphics::present(ctx)
    }
}