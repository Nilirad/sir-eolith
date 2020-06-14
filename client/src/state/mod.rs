mod message_handling;
mod draw;
mod controller;

use crate::ggez_prelude::*;
use crate::connection::{Op, ServerMsg, Connection, ConnectionState};
use crate::components::*;
use crate::consts::*;
use std::time::Instant;
use legion::prelude::*;
use legion::borrow::RefMut;
use derive_more::{Add, AddAssign};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Add, AddAssign)]
pub struct Milliseconds(u128);

#[derive(Debug, Copy, Clone)]
enum PingStatus {
    WaitingResponse,
    GotResponse(Milliseconds),
}

pub struct State<'manager> {
    connection: &'manager mut dyn Connection,
    world: World,
    controller: controller::SnakeController,
    ping_status: PingStatus,
    last_update: Instant,
    player_set: bool,
    grd: f32,
    sector_size: f32,
}

impl<'manager> State<'manager> {
    const PING_TRESHOLD: Milliseconds = Milliseconds(250);

    pub fn new(connection: &'manager mut dyn Connection) -> Self {
        Self {
            connection,
            world: World::new(),
            controller: controller::SnakeController::new(),
            ping_status: PingStatus::GotResponse(Milliseconds(0)),
            last_update: Instant::now(),
            player_set: false,
            grd: 16384.0, // TODO: magic number.
            sector_size: 480.0, // TODO: magic number.
        }
    }

    fn handle_server_message(&mut self, message: ServerMsg) {
        if let Ok(opcode) = message.opcode() {
            match opcode {
                Op::LoginInfo => self.handle_login_info(message),
                Op::SetupGame => self.handle_setup_game(message),
                Op::GameOver => self.handle_game_over(),
                Op::SnakeAction => self.handle_snake_action(message),
                Op::PingResponse => self.handle_ping_response(),
                Op::PosAbs | Op::GrowAbs | Op::PosRel | Op::GrowRel
                    => self.handle_positioning_and_growth(message),
                Op::Shrink => self.handle_shrink(message),
                Op::RemoveSectorFood => self.handle_remove_sector_food(message),
                Op::LoadSectorFood => self.handle_load_sector_food(message),
                Op::SnakeFood | Op::SpawnFood => self.handle_load_single_food(message),
                Op::EatFood => self.handle_eat_food(message),
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

    fn find_snake_with_id(&self, id: Id) -> Option<Entity> {
        let query = <(Read<Id>,)>::query();
        for (entity, (snake_id,)) in query.iter_entities(&self.world) {
            if *snake_id == id {
                return Some(entity);
            }
        }
        None
    }

    fn get_snake_components(&mut self, id: Id) -> Option<(RefMut<Pos>, RefMut<SnakeSegments>)> {
        let query = <(Read<Id>, Write<Pos>, Write<SnakeSegments>)>::query();
        for (snake_id, pos, segments) in query.iter_mut(&mut self.world) {
            if *snake_id == id {
                return Some((pos, segments));
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
            let sector_x = (food_pos.0.x / self.sector_size).floor() as u8;
            let sector_y = (food_pos.0.y / self.sector_size).floor() as u8;

            if sector_x == world_sector.x && sector_y == world_sector.y {
                result.push(food);
            }
        }

        result
    }
}

impl<'manager> EventHandler for State<'manager> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let delta = self.time_since_last_update();
        
        for message in self.connection.poll_messages() {
            self.handle_server_message(message);
        }

        self.server_ping(delta);

        if self.connection.state() == ConnectionState::Playing {
            self.controller.move_snake(delta, self.connection);
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

    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        self.connection.close();
        false
    }
}