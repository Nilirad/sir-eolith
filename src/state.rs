use crate::ggez_prelude::*;
use crate::ws::{ConnectionState, WebSocket};
use crate::consts::*;
use crate::utils::decrypt_message;

pub struct State {
    connection: WebSocket,
}

const LOGIN_REQUEST_MSG: [u8; 1] = [0x63];
const PROVISIONAL_LGBA_MSG: [u8; 4] = [115, 10, 0, 0];

impl State {
    pub fn new() -> Self {
        Self {
            connection: WebSocket::new(SERVER_URL.to_owned()),
        }
    }
}

impl EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let mut messages = self.connection.poll_messages();
        
        match self.connection.state {
            ConnectionState::Uninitialized => {
                self.connection.send_message(LOGIN_REQUEST_MSG.to_vec());
                println!("Sent login request.");
                self.connection.state = ConnectionState::SentLoginRequest;
            }
            ConnectionState::SentLoginRequest => {
                if !messages.is_empty() {
                    debug_assert_eq!(messages.len(), 1);
                    if let Some(message) = messages.pop_front() {
                        debug_assert_eq!(message[2], '6' as u8);
                        println!("Got login info.");
                        self.connection.send_message(decrypt_message(message));
                        self.connection.send_message(PROVISIONAL_LGBA_MSG.to_vec());
                        println!("Sent login credentials,");
                        self.connection.state = ConnectionState::SentIdba;
                    }
                }
            }
            ConnectionState::SentIdba => {
                if let Some(message) = messages.pop_front() {
                    debug_assert_eq!(message[2], 'a' as u8);
                    println!("Got setup game info.");
                    self.connection.state = ConnectionState::Playing;
                }
            }
            ConnectionState::Playing => {

            }
            ConnectionState::Disconnected => {

            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
        graphics::present(ctx)
    }
}