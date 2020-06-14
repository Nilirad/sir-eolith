use super::{State, PingStatus};
use crate::connection::{ClientMsg, ServerMsg, Op, ConnectionState};
use crate::types::*;
use crate::nalgebra_prelude::*;

impl<'connection> State<'connection> {
    pub fn handle_login_info(&mut self, message: ServerMsg) {
        const PROVISIONAL_LGBA_MSG: [u8; 4] = [115, 10, 0, 0];

        self.connection.send(decrypt_message(message));
        self.connection.send(PROVISIONAL_LGBA_MSG.to_vec());
    }

    pub fn handle_setup_game(&mut self, mut message: ServerMsg) {
        self.grd = message.read_u24() as f32;
        let _mscps = message.read_u16();
        self.sector_size = message.read_u16() as f32;
        
        info!("Logged into server.");
        self.connection.set_state(ConnectionState::Playing);
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

    pub fn handle_remove_sector_food(&mut self, mut message: ServerMsg) {
        let world_sector = na::Point2::new(
            message.read_u8(),
            message.read_u8(),
        );

        for food in self.find_foods_in_sector(world_sector) {
            self.world.delete(food);
        }
    }

    pub fn handle_load_sector_food(&mut self, mut message: ServerMsg) {
        let mut foods = Vec::new();
        while !message.has_reached_end() {
            let _unknown = message.read_u8();
            let pos = Pos::new(
                message.read_u16() as f32,
                message.read_u16() as f32,
            );
            let _size = message.read_u8() as f32 / 5.0;
            
            foods.push((pos,));
        }

        self.world.insert((FoodTag,), foods);
    }

    pub fn handle_load_single_food(&mut self, mut message: ServerMsg) {
        let _unknown = message.read_u8();
        if message.len() > 7 {
            let pos = Pos::new(
                message.read_u16() as f32,
                message.read_u16() as f32,
            );
            let _size = message.read_u8() as f32 / 5.0;

            self.world.insert((FoodTag,), vec![(pos,)]);

        } else {
            warn!("Does this even happen?");
        }
    }

    pub fn handle_eat_food(&mut self, mut message: ServerMsg) {
        let pos = Pos::new(
            message.read_u16() as f32,
            message.read_u16() as f32,
        );
        
        if let Some(food) = self.find_food(pos) {
            self.world.delete(food);
        } else {
            warn!("Food ({}, {}) not found.", pos.0.x, pos.0.y);
        }
        
    }
}

pub fn decrypt_message(message: ServerMsg) -> ClientMsg {
    /// Index of the first message byte containing data.
    const MESSAGE_PAYLOAD_START: usize = 3;

    let mut javascript_code = Vec::<u8>::new();
    let mut d = 0u8;
    let mut e = 23;
    
    for (i, byte) in message.into_iter().skip(MESSAGE_PAYLOAD_START).enumerate() {
        let mut b = byte as i32;
        if b <= 96 {
            b += 32;
        }
        b = (b - 97 - e) % 26;
        if b < 0 {
            b += 26;
        }
        d *= 16;
        d += b as u8;
        e += 17;

        if i % 2 == 1 {
            javascript_code.push(d);
            d = 0;
        }
    }

    let idba = (&javascript_code[7..=30]).to_vec();
    let mut b = 0;
    idba.iter().enumerate().map(
        |(i, byte)| {
            let mut d = 65;
            let mut a = *byte as i32;
            if a >= 97 {
                d += 32;
                a -= 32;
            }
            a -= 65;
            if i == 0 {
                b = 2 + a;
            }
            e = a + b;
            e = e % 26;
            b += 3 + a;
            (e + d) as u8
        }
    )
    .collect()
}