use crate::ggez_prelude::*;
use crate::ws::{ConnectionState, WebSocket};
use crate::consts::*;
use crate::message::{ClientMsg, Op, ServerMsg};
use crate::utils::decrypt_message;

pub struct State {
    connection: WebSocket,
}

const PROVISIONAL_LGBA_MSG: [u8; 4] = [115, 10, 0, 0];

impl State {
    pub fn new() -> Self {
        Self {
            connection: WebSocket::new(SERVER_URL.to_owned()),
        }
    }

    fn handle_server_message(&mut self, message: ServerMsg) {
        if let Ok(opcode) = message.opcode() {
            match opcode {
                Op::LoginInfo => {
                    self.connection.send_message(decrypt_message(message));
                    self.connection.send_message(PROVISIONAL_LGBA_MSG.to_vec());
                }
                Op::SetupGame => {
                    println!("Login successful!");
                    self.connection.state = ConnectionState::Playing;
                }
                Op::GameOver => {}
                Op::SnakeAction => {}
                Op::PosAbs | Op::GrowAbs | Op::PosRel | Op::GrowRel
                    => {}
                Op::Shrink => {}
            }    
        }
    }
}

impl EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        for message in self.connection.poll_messages() {
            self.handle_server_message(message);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
        graphics::present(ctx)
    }
}