mod message_handling;
mod draw;

use crate::ggez_prelude::*;
use crate::connection::{Op, ServerMsg, Connection, ConnectionState};
use crate::types::*;
use crate::consts::*;
use crate::controller::SnakeController;
use std::time::Instant;
use legion::prelude::*;
use legion::borrow::RefMut;
use bit_vec::BitVec;

struct Ping {
    waiting_response: bool,
    since: Instant,
    lag_multiplier: f32,
}

impl Ping {
    fn new(waiting_response: bool) -> Self {
        Self {
            waiting_response,
            since: Instant::now(),
            lag_multiplier: 1.0,
        }
    }

    fn switch(&mut self) {
        self.waiting_response = !self.waiting_response;
        self.since = Instant::now();
    }

    fn lagging(&self) -> bool {
        self.waiting_response && (Instant::now() - self.since).as_millis() > 420
    }

    fn can_send_ping(&self) -> bool {
        !self.waiting_response && (Instant::now() - self.since).as_millis() > 250
    }

    fn update_lag_multiplier(&mut self) {
        self.lag_multiplier = if self.lagging() {
            (self.lag_multiplier * 0.85).max(0.01)
        } else {
            (self.lag_multiplier + 0.05).min(1.0)
        };
    }
}

/// Game zoom. If `None`, zoom is automatic. If `Some`, zoom is manual, defined by the
/// inner value.
struct Zoom(Option<f32>);

impl Zoom {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn factor(&self, body_length: f32) -> f32 {
        match self.0 {
            Some(factor) => factor,
            None => 0.64285 + 0.514285714 / 1.0f32.max((body_length + 16.0) / 36.0),
        }
    }
}

struct Params {
    world_radius: f32,
    sector_size: f32,
    spangdv: f32,
    mamu: f32,
    prey_mamu: f32,
    cst: f32,
    fmlts: Vec<f32>,
    fpsls: Vec<f32>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            world_radius: 16384.0,
            sector_size: 480.0,
            spangdv: 4.8,
            mamu: 0.033,
            prey_mamu: 0.028,
            cst: 0.43,
            fmlts: Vec::new(),
            fpsls: Vec::new(),
        }
    }
}

struct Stats {
    rank_text: Text,
    leaderboard: Vec<(Text, Text, Text)>,
    score: u32,
    score_text: Text,
    minimap_data: BitVec,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            rank_text: Text::new("Your rank:"),
            leaderboard: Vec::new(),
            score: u32::default(),
            score_text: Text::new("Score:"),
            minimap_data: BitVec::from_elem(MINIMAP_AREA, false),
        }
    }
}

impl Stats {
    pub fn add_to_leaderboard(&mut self, nickname: String, score: u32) {
        let position_text = Text::new(format!("{}.", (self.leaderboard.len() + 1)));
        let name_text = Text::new(nickname);
        let score_text = Text::new(score.to_string());
        
        self.leaderboard.push((position_text, name_text, score_text));
    }

    pub fn update_rank(&mut self, your_rank: u16, player_count: u16) {
        self.rank_text = Text::new(format!("Your rank: {} / {}", your_rank, player_count));
    }

    pub fn update_score(&mut self, score: u32) {
        self.score = score;
        self.score_text = Text::new(format!("Score: {}", score));
    }

    pub fn clear_minimap(&mut self) {
        self.minimap_data.set_all();
        self.minimap_data.negate();
    }

    pub fn set_minimap_pixel(&mut self, pen_position: usize) {
        self.minimap_data.set(pen_position, true);
    }
}

pub struct State<'connection> {
    connection: &'connection mut dyn Connection,
    world: World,
    controller: SnakeController,
    ping_status: Ping,
    last_update: Instant,
    player_set: bool,
    zoom: Zoom,
    params: Params,
    stats: Stats,
}

impl<'connection> State<'connection> {
    pub fn new(connection: &'connection mut dyn Connection) -> Self {
        Self {
            connection,
            world: World::new(),
            controller: SnakeController::new(),
            ping_status: Ping::new(false),
            last_update: Instant::now(),
            player_set: false,
            zoom: Zoom::new(),
            params: Params::default(),
            stats: Stats::default(),
        }
    }

    fn handle_server_message(&mut self, message: ServerMsg) {
        if let Ok(opcode) = message.opcode() {
            match opcode {
                Op::LoginInfo => self.handle_login_info(message),
                Op::SetupGame => self.handle_setup_game(message),
                Op::Angle1 | Op::Angle2 | Op::Angle3 | Op::Angle4 | Op::Angle5
                    => self.handle_angle(message, opcode),
                Op::GameOver => self.handle_game_over(),
                Op::UpdateFullness => self.handle_update_fullness(message),
                Op::SnakeAction => self.handle_snake_action(message),
                Op::PingResponse => self.handle_ping_response(),
                Op::UpdateMinimap => self.handle_update_minimap(message),
                Op::PosAbs | Op::GrowAbs | Op::PosRel | Op::GrowRel
                    => self.handle_positioning_and_growth(message),
                Op::UpdateStats => self.handle_update_stats(message),
                Op::Shrink => self.handle_shrink(message),
                Op::RemoveSectorFood => self.handle_remove_sector_food(message),
                Op::LoadSectorFood => self.handle_load_sector_food(message),
                Op::SnakeFood | Op::SpawnFood => self.handle_load_single_food(message),
                Op::EatFood => self.handle_eat_food(message),
                Op::UpdatePrey => self.handle_update_prey(message),
                Op::PreyAction => self.handle_prey_action(message),
            }    
        }
    }

    fn server_ping(&mut self) {
        const PING_MESSAGE: [u8; 1] = [0xFB];

        if self.ping_status.can_send_ping() {
            self.connection.send(PING_MESSAGE.to_vec());
            self.ping_status.switch();
        }
    }

    fn score(&self, snake_point_count: usize, fullness: Fullness) -> u32 {
        let fpsl = self.params.fpsls[snake_point_count];
        let fmlt = self.params.fmlts[snake_point_count];
        (15.0 * (fpsl + fullness / fmlt - 1.0) - 5.0).floor() as u32
    }

    fn find_snake_with_id(&self, id: Id) -> Option<Entity> {
        let query = <(Read<Id>,)>::query();
        for (entity, (snake_id,)) in query.iter_entities(&self.world) {
            if *snake_id == id {
                return Some(entity);
            }
        }
        None
    }

    fn find_prey_with_id(&self, id: Id) -> Option<Entity> {
        let query = <(Read<Id>,)>::query()
            .filter(tag::<PreyTag>());
        for (entity, (prey_id,)) in query.iter_entities(&self.world) {
            if *prey_id == id {
                return Some(entity);
            }
        }
        None
    }

    fn get_snake_components(&mut self, id: Id) -> Option<(RefMut<Pos>, RefMut<Body>, RefMut<Movement>)> {
        let query = <(Read<Id>, Write<Pos>, Write<Body>, Write<Movement>)>::query();
        for (snake_id, pos, body, movement) in query.iter_mut(&mut self.world) {
            if *snake_id == id {
                return Some((pos, body, movement));
            }
        }
        None
    }

    fn find_food(&self, pos: Pos) -> Option<Entity> {
        let query = <(Read<Pos>,)>::query()
            .filter(tag::<FoodTag>());
        for (food, (food_pos,)) in query.iter_entities(&self.world) {
            if *food_pos == pos {
                return Some(food);
            }
        }
        None
    }

    fn find_foods_in_sector(&self, world_sector: na::Point2<u8>) -> Vec<Entity> {
        let query = <(Read<Pos>,)>::query()
            .filter(tag::<FoodTag>());
        let mut result = Vec::new();
        for (food, (food_pos,)) in query.iter_entities(&self.world) {
            let sector_x = (food_pos.x / self.params.sector_size).floor() as u8;
            let sector_y = (food_pos.y / self.params.sector_size).floor() as u8;

            if sector_x == world_sector.x && sector_y == world_sector.y {
                result.push(food);
            }
        }

        result
    }

    fn interpolate_snakes(&mut self, vfr: f32) {
        let query = <(Write<Pos>, Read<Body>, Write<Movement>)>::query();
        for (mut pos, body, mut movement) in query.iter_mut(&mut self.world) {
            let delta_angle = self.params.mamu * vfr * body.scang() * movement.spang(self.params.spangdv) * movement.direction.value();
            *pos += movement.interpolate(delta_angle, vfr);
        }
    }

    fn interpolate_preys(&mut self, vfr: f32) {
        let base_delta_angle = self.params.prey_mamu * vfr;
        let query = <(Write<Pos>, Write<Movement>)>::query()
            .filter(tag::<PreyTag>());
        for (mut pos, mut movement) in query.iter_mut(&mut self.world) {
            let delta_angle = base_delta_angle * movement.direction.value();
            *pos += movement.interpolate(delta_angle, vfr);
        }
    }
}

impl<'connection> EventHandler for State<'connection> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let now = Instant::now();
        let delta = (now - self.last_update).as_millis();
        self.last_update = now;
        
        for message in self.connection.poll_messages() {
            self.handle_server_message(message);
        }

        self.server_ping();

        if self.connection.state() == ConnectionState::Playing {
            self.controller.move_snake(self.connection);
        }

        self.ping_status.update_lag_multiplier();
        let vfr = (delta as f32 / 8.0).max(1.56).min(5.0) * self.ping_status.lag_multiplier;

        self.interpolate_snakes(vfr);
        self.interpolate_preys(vfr);

        // TODO: PLAYER SCORE (refactor!)
        let query = <(Read<Body>,)>::query()
            .filter(tag_value(&PlayerTag(PlayerType::You)));
        for (body,) in query.iter(&self.world) {
            let score = self.score(body.points.len(), body.fullness);
            if score != self.stats.score {
                self.stats.update_score(score);
            }
        }
        

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
        self.draw_game(ctx)?;
        
        graphics::present(ctx)
    }

    /// Handles mouse movement, updating the mouse position.
    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.controller.set_mouse_pos(x - (WINDOW_WIDTH / 2.0), y - (WINDOW_HEIGHT / 2.0));
    }

    /// Handles mouse button presses.
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        self.controller.set_mouse_pressed(button == MouseButton::Left);
    }

    /// Handles mouse buttons being lifted.
    fn mouse_button_up_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        self.controller.set_mouse_pressed(false);
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        self.zoom.0 = match self.zoom.0 {
            Some(factor) => Some((factor + 0.1 * y).max(0.3).min(2.0)),
            None => {
                let query = <(Read<Body>,)>::query()
                    .filter(tag_value(&PlayerTag(PlayerType::You)));
                    let mut result = None;
                    for (body,) in query.iter(&mut self.world) {
                        let cur_zoom = self.zoom.factor(body.length());
                        result = Some((cur_zoom + 0.1 * y).min(0.5));
                    }

                    result
            },
        }
    }

    fn key_down_event(
            &mut self,
            _ctx: &mut Context,
            keycode: KeyCode,
            _keymods: KeyMods,
            _repeat: bool,
        ) {
        match keycode {
            KeyCode::Z => self.zoom.0 = None,
            _ => (),
        }
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        self.connection.close();
        false
    }
}