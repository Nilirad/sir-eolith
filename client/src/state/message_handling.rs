use super::State;
use crate::connection::{ClientMsg, ServerMsg, Op, ConnectionState};
use crate::types::*;
use crate::nalgebra_prelude::*;
use legion::prelude::*;
use ggez::graphics::{Text, TextFragment, Scale};

impl<'connection> State<'connection> {
    pub fn handle_login_info(&mut self, message: ServerMsg) {
        const PROVISIONAL_LGBA_MSG: [u8; 7] = [115, 10, 0, 3, 'b' as u8, 'o' as u8, 'i' as u8];

        self.connection.send(decrypt_message(message));
        self.connection.send(PROVISIONAL_LGBA_MSG.to_vec());
    }

    pub fn handle_setup_game(&mut self, mut message: ServerMsg) {
        self.params.world_radius = message.read_u24() as f32;
        self.mscps(message.read_u16() as usize);
        self.params.sector_size = message.read_u16() as f32;
        message.skip(2); // `sector_count_along_edge` is unused.
        self.params.spangdv = message.read_u8() as f32 / 10.0;
        message.go_to(19);
        self.params.mamu = message.read_u16() as f32 / 1000.0;
        self.params.prey_mamu = message.read_u16() as f32 / 1000.0;
        self.params.cst = message.read_u16() as f32 / 1000.0;
        
        info!("Logged into server.");
        self.connection.set_state(ConnectionState::Playing);
    }

    fn mscps(&mut self, mscps: usize) {
        for i in 0..=mscps {
            self.params.fmlts.push(
                if i < mscps { (1.0 - i as f32 / mscps as f32).powf(2.25) }
                else { self.params.fmlts.last().copied().unwrap() }
            );

            self.params.fpsls.push(
                if i > 0 { 
                    self.params.fpsls.last().copied().unwrap()
                        + 1.0 / self.params.fmlts[i - 1]
                }
                else { 0.0 }
            );
        }

        let last_fmlt = self.params.fmlts.last().copied().unwrap();
        let last_fpsl = self.params.fpsls.last().copied().unwrap();
        for _ in 0..2048 {
            self.params.fmlts.push(last_fmlt);
            self.params.fpsls.push(last_fpsl);
        }
    }

    pub fn handle_angle(&mut self, mut message: ServerMsg, opcode: Op) {
        let id = message.read_id();

        let query = <(Read<Id>, Write<Movement>)>::query();
        for (snake_id, mut movement) in query.iter_mut(&mut self.world) {
            if *snake_id == id {
                match message.len() {
                    8 => {
                        movement.direction = if opcode == Op::Angle1 {
                            Direction::Left
                        } else {
                            Direction::Right
                        };
                        movement.angle = message.read_angle_short();
                        movement.target_angle = message.read_angle_short();
                        movement.speed = message.read_speed_short();
                    }
                    7 => {
                        match opcode {
                            Op::Angle1 => {
                                movement.angle = message.read_angle_short();
                                movement.speed = message.read_speed_short();
                            }
                            Op::Angle2 => {
                                movement.direction = Direction::Left;
                                movement.target_angle = message.read_angle_short();
                                movement.speed = message.read_speed_short();
                            }
                            Op::Angle3 => {
                                movement.direction = Direction::Left;
                                movement.angle = message.read_angle_short();
                                movement.target_angle = message.read_angle_short();
                            }
                            Op::Angle4 => {
                                movement.direction = Direction::Right;
                                movement.target_angle = message.read_angle_short();
                                movement.speed = message.read_speed_short();
                            }
                            Op::Angle5 => {
                                movement.direction = Direction::Right;
                                movement.angle = message.read_angle_short();
                                movement.target_angle = message.read_angle_short();
                            }
                            _ => unreachable!(),
                        }
                    },
                    6 => {
                        match opcode {
                            Op::Angle1 => {
                                movement.angle = message.read_angle_short();
                            }
                            Op::Angle2 => {
                                movement.direction = Direction::Left;
                                movement.target_angle = message.read_angle_short();
                            }
                            Op::Angle3 => {
                                movement.speed = message.read_speed_short();
                            }
                            Op::Angle4 => {
                                movement.direction = Direction::Right;
                                movement.target_angle = message.read_angle_short();
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                } 
                break;
            }
        }
    }

    pub fn handle_game_over(&mut self) {
        self.connection.close(); // TODO: Wait some time before closing.
    }

    pub fn handle_update_fullness(&mut self, mut message: ServerMsg) {
        let id = message.read_id();
        if let Some((_, mut body, _)) = self.get_snake_components(id) {
            body.fullness = message.read_fullness();
        }
    }

    pub fn handle_snake_action(&mut self, mut message: ServerMsg) {
        enum Action { Load, Unload }
        let id = message.read_id();

        let action = if message.len() > 9 { Action::Load } else { Action::Unload };
        match action {
            Action::Load => {
                let angle = message.read_angle();
                message.skip(1); // unused byte
                let target_angle = message.read_angle();
                let speed = message.read_speed();
                let fullness = message.read_fullness();
                message.go_to(18);
                let pos = message.read_pos_6_bytes();

                let nickname = message.read_string();
                let text_fragment = TextFragment::new(nickname).scale(Scale::uniform(50.0));
                let nickname_text = Text::new(text_fragment);
                self.cache.nicknames.insert(id, nickname_text);

                let skin_data_length = message.read_u8();
                let _skin_data = if skin_data_length > 0 {
                    let skin = ();
                    message.skip(skin_data_length as usize);
                    Some(skin)
                } else {
                    None
                };

                let body = message.read_body(fullness);

                let movement = Movement::new(speed, angle, target_angle, Direction::None);

                let tag = if self.cache.player_set {
                    PlayerTag(PlayerType::Other)
                } else {
                    self.cache.player_set = true; // TODO: This side effect is not the best thing...
                    PlayerTag(PlayerType::You)
                };

                self.world.insert(
                    (tag,),
                    vec![(
                        (
                            id,
                            pos,
                            body,
                            movement,
                        )
                    )]
                );
                trace!("Loaded snake {}", id);
            }
            Action::Unload => {
                if let Some(snake) = self.find_snake_with_id(id) {
                    self.world.delete(snake);
                    trace!("Unloaded snake {}", id);
                } else {
                    warn!("[snake unload action] Snake {} not found.", id);
                }
            }
        }
    }

    pub fn handle_ping_response(&mut self) {
        self.ping_status.switch();
        trace!("Pong");
    }

    pub fn handle_update_minimap(&mut self, mut message: ServerMsg) {
        self.stats.clear_minimap();
        let mut pen_position = 0;
        while !message.has_reached_end() && pen_position < crate::consts::MINIMAP_AREA {
            let byte = message.read_u8();
            if byte >= 128 {
                pen_position += (byte - 128) as usize;
            } else {
                let mut mask = 0b_0100_0000_u8;
                while mask > 0b_0000_0000 {
                    if (byte & mask) > 0 {
                        self.stats.set_minimap_pixel(pen_position);
                    }
                    pen_position += 1;
                    mask >>= 1;
                }
            }
        }
    }

    pub fn handle_positioning_and_growth(&mut self, mut message: ServerMsg) {
        let growing = message.opcode() == Ok(Op::GrowAbs) || message.opcode() == Ok(Op::GrowRel);
        let absolute = message.opcode() == Ok(Op::PosAbs) || message.opcode() == Ok(Op::GrowAbs);
        let id = message.read_id();

        let lag_multiplier = self.ping_status.lag_multiplier;
        if let Some((mut pos, mut body, movement)) = self.get_snake_components(id) {
            let new_pos = if absolute {
                message.read_pos_4_bytes()
            } else {
                body.head_pos() + message.read_pos_delta()
            };
            body.add_point(new_pos);

            pos.x = new_pos.x + movement.angle.cos() * lag_multiplier;
            pos.y = new_pos.y + movement.angle.sin() * lag_multiplier;

            match growing {
                true => body.fullness = message.read_fullness(),
                false => body.shrink(),
            }

            // TODO: Horrible, non-idiomatic code, directly translated from the horrible
            // JavaScript source.
            let mut last = None;
            let mut w = 0.0;
            let mut n = 0usize;
            for point in body.points.iter_mut().rev().skip(2) {
                last = match last {
                    None => Some(point),
                    Some(last_point) => {
                        n += 1;
                        if n <= 4 {
                            w = 0.43 * n as f32 / 4.0;
                        }
                        *point += (*last_point - *point) * w;
                        Some(point)
                    }
                }
            }
        } else {
            warn!("[pos/grow] Snake {} not found.", id);
        }
    }

    pub fn handle_update_stats(&mut self, mut message: ServerMsg) {
        message.skip(1); // Your rank in leaderboard, redundant.
        let your_rank = message.read_u16();
        let player_count = message.read_u16();
        self.stats.update_rank(your_rank, player_count);
        self.stats.leaderboard.clear();
        while !message.has_reached_end() {
            let snake_point_count = message.read_u16() as usize;
            let fullness = message.read_fullness();
            let _text_color = message.read_u8() % 9;
            let nickname = message.read_string();
            self.stats.add_to_leaderboard(nickname, self.score(snake_point_count, fullness));
        }
    }

    pub fn handle_shrink(&mut self, mut message: ServerMsg) {
        let id = message.read_id();
        if let Some((_pos, mut body, _)) = self.get_snake_components(id) {
            if message.len() >= 7 {
                body.fullness = message.read_fullness();
            }
            body.shrink();
        } else {
            warn!("[shrink] Snake {} not found.", id);
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
            let pos = message.read_pos_4_bytes();
            let size = message.read_size();
            
            foods.push((pos, size));
        }

        self.world.insert((FoodTag,), foods);
    }

    pub fn handle_load_single_food(&mut self, mut message: ServerMsg) {
        let _unknown = message.read_u8();
        if message.len() > 7 {
            let pos = message.read_pos_4_bytes();
            let size = message.read_size();

            self.world.insert((FoodTag,), vec![(pos, size)]);

        } else {
            warn!("Does this even happen?");
        }
    }

    pub fn handle_eat_food(&mut self, mut message: ServerMsg) {
        let pos = message.read_pos_4_bytes();
        
        if let Some(food) = self.find_food(pos) {
            self.world.delete(food);
        } else {
            warn!("Food ({}, {}) not found.", pos.x, pos.y);
        }
        
    }

    pub fn handle_update_prey(&mut self, mut message: ServerMsg) {
        let id = message.read_id();
        let query = <(Read<Id>, Write<Pos>, Write<Movement>)>::query()
            .filter(tag::<PreyTag>());
        for (prey_id, mut pos, mut movement) in query.iter_mut(&mut self.world) {
            if *prey_id == id {
                let displacement = self.ping_status.lag_multiplier * movement.speed / 4.0;
                let msg_pos = message.read_prey_pos();
                *pos = Pos::new(
                    msg_pos.x + movement.angle.cos() * displacement,
                    msg_pos.y + movement.angle.sin() * displacement,
                );

                match message.len() {
                    18 => {
                        movement.direction = message.read_direction();
                        movement.angle = message.read_angle();
                        movement.target_angle = message.read_angle();
                        movement.speed = message.read_speed();
                    }
                    14 => {
                        movement.angle = message.read_angle();
                        movement.speed = message.read_speed();
                    }
                    15 => {
                        movement.direction = message.read_direction();
                        movement.target_angle = message.read_angle();
                        movement.speed = message.read_speed();
                    }
                    16 => {
                        movement.direction = message.read_direction();
                        movement.angle = message.read_angle();
                        movement.target_angle = message.read_angle();
                    }
                    12 => {
                        movement.angle = message.read_angle();
                    }
                    13 => {
                        movement.direction = message.read_direction();
                        movement.target_angle = message.read_angle();
                    }
                    11 => {
                        movement.speed = message.read_speed();
                    }
                    _ => unreachable!(),
                }

                break;
            }
        }
    }

    pub fn handle_prey_action(&mut self, mut message: ServerMsg) {
        const UNLOAD: usize = 5;
        const EAT: usize = 7;
        
        let id = message.read_id();

        match message.len() {
            UNLOAD | EAT => { // TODO: Treat EAT as separate case when rendering gravitation.
                if let Some(prey) = self.find_prey_with_id(id) {
                    self.world.delete(prey);
                }
            }
            _ => {
                let _color = message.read_u8();
                let pos = message.read_pos_6_bytes();
                let size = message.read_size();
                let direction = message.read_direction();
                let target_angle = message.read_angle();
                let angle = message.read_angle();
                let speed = message.read_speed();

                let movement = Movement::new(speed, angle, target_angle, direction);

                self.world.insert(
                    (PreyTag,),
                    vec![(
                        id,
                        pos,
                        size,
                        movement,
                    )]
                );
            }
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