use super::{State, PingStatus, Milliseconds};
use crate::utils::decrypt_message;
use crate::ws::ConnectionState;
use crate::message::{ServerMsg, Op};
use crate::snake::SnakeSegment;
use crate::components::*;

impl State {
    

    pub fn handle_login_info(&mut self, message: ServerMsg) {
        const PROVISIONAL_LGBA_MSG: [u8; 4] = [115, 10, 0, 0];

        self.connection.send(decrypt_message(message));
        self.connection.send(PROVISIONAL_LGBA_MSG.to_vec());
    }

    pub fn handle_setup_game(&mut self) {
        info!("Logged into server.");
        self.connection.state = ConnectionState::Playing;
    }

    pub fn handle_game_over(&mut self) {
        self.connection.close(); // TODO: Wait some time before closing.
    }

    pub fn handle_snake_action(&mut self, mut message: ServerMsg) {
        enum Action { Load, Unload }
        let id = Id(message.read_u16());

        let action = if message.len() > 9 { Action::Load } else { Action::Unload };
        match action {
            Action::Load => {
                message.go_to(18);
                let pos = Pos::new(
                    message.read_u24() as f32,
                    message.read_u24() as f32
                );
                
                let nickname_length = message.read_u8();
                let _nickname = if nickname_length > 0 {
                    let nick = "TODO".to_owned();
                    message.skip(nickname_length as usize);
                    nick
                } else {
                    "".to_owned()
                };

                let skin_data_length = message.read_u8();
                let _skin_data = if skin_data_length > 0 {
                    let skin = ();
                    message.skip(skin_data_length as usize);
                    Some(skin)
                } else {
                    None
                };

                // TODO: Rewrite in an idiomatic way
                let mut section_x = 0.0;
                let mut section_y = 0.0;
                let mut segments = SnakeSegments::new();
                let mut first = true;
                while !message.has_reached_end() {
                    if first {
                        first = false;
                        section_x = message.read_u24() as f32 / 5.0;
                        section_y = message.read_u24() as f32 / 5.0;
                    } else {
                        section_x += (message.read_u8() as i32 - 127) as f32 / 2.0;
                        section_y += (message.read_u8() as i32 - 127) as f32 / 2.0;
                    }
                    segments.push(SnakeSegment::new(section_x, section_y));
                }

                let tag = if self.player_set {
                    PlayerTag(PlayerType::Other)
                } else {
                    self.player_set = true; // TODO: This side effect is not the best thing...
                    PlayerTag(PlayerType::You)
                };

                self.world.insert(
                    (tag,),
                    vec![(
                        (
                            id,
                            pos,
                            segments,
                        )
                    )]
                );
                trace!("Loaded snake {}", id.0);
            }
            Action::Unload => {
                if let Some(snake) = self.find_snake_with_id(id) {
                    self.world.delete(snake);
                    trace!("Unloaded snake {}", id.0);
                } else {
                    warn!("[snake unload action] Snake {} not found.", id.0);
                }
            }
        }
    }

    pub fn handle_ping_response(&mut self) {
        self.ping_status = PingStatus::GotResponse(Milliseconds(0));
        trace!("Pong");
    }

    pub fn handle_positioning_and_growth(&mut self, mut message: ServerMsg) {
        let growing = message.opcode() == Ok(Op::GrowAbs) || message.opcode() == Ok(Op::GrowRel);
        let absolute = message.opcode() == Ok(Op::PosAbs) || message.opcode() == Ok(Op::GrowAbs);
        let id = Id(message.read_u16());

        if let Some((mut pos, mut segments)) = self.get_snake_components(id) {
            if absolute {
                pos.0.x = message.read_u16() as f32; // TODO: Abstract away to remove 0's.
                pos.0.y = message.read_u16() as f32;
            } else {
                pos.0.x += message.read_u8() as f32 - 128.0;
                pos.0.y += message.read_u8() as f32 - 128.0;
            }
            segments.push(SnakeSegment::new(pos.0.x, pos.0.y));

            if !growing {
                segments.remove_tail();
            }
        } else {
            warn!("[pos/grow] Snake {} not found.", id.0);
        }
    }

    pub fn handle_shrink(&mut self, mut message: ServerMsg) {
        let id = Id(message.read_u16());
        if let Some((_pos, mut segments)) = self.get_snake_components(id) {
            segments.remove_tail();
        } else {
            warn!("[shrink] Snake {} not found.", id.0);
        }
    }
}