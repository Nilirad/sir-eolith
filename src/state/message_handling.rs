use super::{State, PingStatus, Milliseconds};
use crate::utils::decrypt_message;
use crate::ws::ConnectionState;
use crate::message::ServerMsg;

impl State {
    

    pub fn handle_login_info(&mut self, message: ServerMsg) {
        const PROVISIONAL_LGBA_MSG: [u8; 4] = [115, 10, 0, 0];

        self.connection.send(decrypt_message(message));
        self.connection.send(PROVISIONAL_LGBA_MSG.to_vec());
    }

    pub fn handle_setup_game(&mut self) {
        println!("Login successful!");
        self.connection.state = ConnectionState::Playing;
    }

    pub fn handle_game_over(&mut self) {
        
    }

    pub fn handle_snake_action(&mut self) {
        
    }

    pub fn handle_ping_response(&mut self) {
        self.ping_status = PingStatus::GotResponse(Milliseconds(0));
    }

    pub fn handle_positioning_and_growth(&mut self) {
        
    }

    pub fn handle_shrink(&mut self) {
        
    }
}