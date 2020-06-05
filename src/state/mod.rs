mod message_handling;

use crate::ggez_prelude::*;
use crate::ws::WebSocket;
use crate::message::{Op, ServerMsg};
use std::time::Instant;
use derive_more::Add;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Add)]
struct Milliseconds(u128);

#[derive(Debug, Copy, Clone)]
enum PingStatus {
    WaitingResponse,
    GotResponse(Milliseconds),
}

pub struct State {
    connection: WebSocket,
    ping_status: PingStatus,
    last_update: Instant,
}

impl State {
    const PING_TRESHOLD: Milliseconds = Milliseconds(250);

    pub fn new(connection: WebSocket) -> Self {
        Self {
            connection,
            ping_status: PingStatus::GotResponse(Milliseconds(0)),
            last_update: Instant::now(),
        }
    }

    fn handle_server_message(&mut self, message: ServerMsg) {
        if let Ok(opcode) = message.opcode() {
            match opcode {
                Op::LoginInfo => self.handle_login_info(message),
                Op::SetupGame => self.handle_setup_game(),
                Op::GameOver => self.handle_game_over(),
                Op::SnakeAction => self.handle_snake_action(),
                Op::PingResponse => self.handle_ping_response(),
                Op::PosAbs | Op::GrowAbs | Op::PosRel | Op::GrowRel
                    => self.handle_positioning_and_growth(),
                Op::Shrink => self.handle_shrink(),
            }    
        }
    }

    fn time_since_last_update(&mut self) -> Milliseconds {
        let now = Instant::now();
        let delta = (now - self.last_update).as_millis();
        self.last_update = now;

        Milliseconds(delta)
    }

    fn server_ping(&mut self, delta: Milliseconds) {
        const PING_MESSAGE: [u8; 1] = [0xFB];

        if let PingStatus::GotResponse(since_last_pong) = self.ping_status {
            let elapsed = since_last_pong + delta;
            if elapsed >= Self::PING_TRESHOLD {
                self.connection.send(PING_MESSAGE.to_vec());
                self.ping_status = PingStatus::WaitingResponse;
            } else {
                self.ping_status = PingStatus::GotResponse(elapsed);
            }
        }
    }
}

impl EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let delta = self.time_since_last_update();
        
        for message in self.connection.poll_messages() {
            self.handle_server_message(message);
        }

        self.server_ping(delta);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
        graphics::present(ctx)
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        self.connection.close();
        false
    }
}