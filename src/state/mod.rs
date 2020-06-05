mod message_handling;

use crate::ggez_prelude::*;
use crate::ws::WebSocket;
use crate::message::{Op, ServerMsg};
use crate::components::*;
use std::time::Instant;
use legion::prelude::*;
use legion::borrow::RefMut;
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
    world: World,
    ping_status: PingStatus,
    last_update: Instant,
    player_set: bool,
}

impl State {
    const PING_TRESHOLD: Milliseconds = Milliseconds(250);

    pub fn new(connection: WebSocket) -> Self {
        Self {
            connection,
            world: World::new(),
            ping_status: PingStatus::GotResponse(Milliseconds(0)),
            last_update: Instant::now(),
            player_set: false,
        }
    }

    fn handle_server_message(&mut self, message: ServerMsg) {
        if let Ok(opcode) = message.opcode() {
            match opcode {
                Op::LoginInfo => self.handle_login_info(message),
                Op::SetupGame => self.handle_setup_game(),
                Op::GameOver => self.handle_game_over(),
                Op::SnakeAction => self.handle_snake_action(message),
                Op::PingResponse => self.handle_ping_response(),
                Op::PosAbs | Op::GrowAbs | Op::PosRel | Op::GrowRel
                    => self.handle_positioning_and_growth(message),
                Op::Shrink => self.handle_shrink(message),
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

    fn get_snake_components(&mut self, id: Id) -> Option<(RefMut<'_, Pos>, RefMut<'_, SnakeSegments>)> {
        let query = <(Read<Id>, Write<Pos>, Write<SnakeSegments>)>::query();
        for (snake_id, pos, segments) in query.iter_mut(&mut self.world) {
            if *snake_id == id {
                return Some((pos, segments));
            }
        }
        None
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