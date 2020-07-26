//! Allows the user to send snake movement requests to the server.

use crate::nalgebra_prelude::*;
use crate::connection::Connection;
use std::f32::consts::PI;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Mouse {
    /// The x and y coordinates of the mouse.
    pub pos: Vec2,
    /// The left button of the mouse: true if pressed, false otherwise.
    pub pressed: bool,
}

impl Mouse {
    fn new() -> Self {
        Self {
            pos: Vec2::new(0.0, 0.0),
            pressed: false,
        }
    }
}

/// The message that tells the server to make the player's snake to boost.
const START_BOOST_MESSAGE: [u8; 1] = [253];
/// The message that tells the server to make the player's snake to stop boosting.
const STOP_BOOST_MESSAGE: [u8; 1] = [254];

/// Holds data relevant to snake movement.
pub struct SnakeController {
    /// The mouse state.
    mouse: Mouse,
    /// `true` if the mouse position changed over the last update.
    mouse_pos_changed: bool,
    /// `true` if the mouse left button state changed over the last update.
    mouse_pressed_changed: bool,
    /// The cooldown time for sending a steering message to the server.
    steer_instant: Instant,
    /// The cooldown time for sending a boost message to the server.
    boost_instant: Instant,
    /// The last angle message sent to the server.
    last_send_angle: u8, // TODO: Make option, for initialization
}

impl SnakeController {
    /// Creates a new controller.
    pub fn new() -> Self {
        Self {
            mouse: Mouse::new(),
            mouse_pos_changed: false,
            mouse_pressed_changed: false,
            steer_instant: Instant::now(),
            boost_instant: Instant::now(),
            last_send_angle: 0, // TODO: Make None, for initialization
        }
    }

    /// Sets the mouse position and checks if it has changed.
    pub fn set_mouse_pos(&mut self, x: f32, y: f32) {
        let new_pos = Vec2::new(x, y);
        if self.mouse.pos != new_pos {
            self.mouse.pos = new_pos;
            self.mouse_pos_changed = true;
        }
    }

    /// Sets the left mouse button press state and checks if it has changed.
    pub fn set_mouse_pressed(&mut self, pressed: bool) {
        if self.mouse.pressed != pressed {
            self.mouse.pressed = pressed;
            self.mouse_pressed_changed = true;
        }
    }

    /// Moves the snake by sending steering and boosting messages to the server.
    pub fn move_snake(&mut self, connection: &mut dyn Connection) {
        let now = Instant::now();
        self.handle_snake_steer(now, connection);
        self.handle_snake_boost(now, connection);
    }

    /// Handles snake steering.
    fn handle_snake_steer(&mut self, now: Instant, connection: &mut dyn Connection) {
        const STEER_COOLDOWN: Duration = Duration::from_millis(100);

        if self.mouse_pos_changed && now >= self.steer_instant {
            self.mouse_pos_changed = false;
            let mouse_angle = angle(self.mouse.pos);
            let send_angle = (251.0 * mouse_angle / (2.0 * PI)).floor() as u8;
            if self.last_send_angle != send_angle {
                self.last_send_angle = send_angle;
                connection.send(vec![send_angle]);
                self.steer_instant = now + STEER_COOLDOWN;
            }
        }
    }

    /// Handles snake boosting.
    fn handle_snake_boost(&mut self, now: Instant, connection: &mut dyn Connection) {
        const BOOST_COOLDOWN: Duration = Duration::from_millis(150);

        if self.mouse_pressed_changed && now >= self.boost_instant {
            self.mouse_pos_changed = false;
            self.boost_instant = now + BOOST_COOLDOWN;
            connection.send(
                if self.mouse.pressed { START_BOOST_MESSAGE.to_vec() }
                else { STOP_BOOST_MESSAGE.to_vec() }
            );
        }
    }
}

/// Returns the angle, in radians, in the range `[0, 2π)`, from the x-axis to the given
/// point. The direction of the angle is determined by the rotation from the x-axis to
/// the y-axis.
fn angle(point: Vec2) -> f32 {
    let (x, y) = (point.x, point.y);

    let angle = y.atan2(x);
    if angle >= 0.0 {
        angle
    } else {
        angle + 2.0 * PI
    }
}