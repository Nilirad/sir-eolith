//! Allows the user to send snake movement requests to the server.

use crate::nalgebra_prelude::*;
use crate::message::Connection;
use super::Milliseconds;
use std::f32::consts::PI;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Mouse {
    /// The x and y coordinates of the mouse.
    pub pos: Point2,
    /// The left button of the mouse: true if pressed, false otherwise.
    pub pressed: bool,
}

impl Mouse {
    fn new() -> Self {
        Self {
            pos: Point2::new(0.0, 0.0),
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
    steer_cooldown: Milliseconds,
    /// The cooldown time for sending a boost message to the server.
    boost_cooldown: Milliseconds,
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
            steer_cooldown: Milliseconds(0),
            boost_cooldown: Milliseconds(0),
            last_send_angle: 0, // TODO: Make None, for initialization
        }
    }

    /// Sets the mouse position and checks if it has changed.
    pub fn set_mouse_pos(&mut self, x: f32, y: f32) {
        let new_pos = Point2::new(x, y);
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
    pub fn move_snake(&mut self, delta: Milliseconds, connection: &mut dyn Connection) {
        
        
        self.handle_snake_steer(delta, connection);
        self.handle_snake_boost(delta, connection);
    }

    /// Handles snake steering.
    fn handle_snake_steer(&mut self, delta: Milliseconds, connection: &mut dyn Connection) {
        const STEER_COOLDOWN: Milliseconds = Milliseconds(100);

        self.steer_cooldown += delta;
        if self.mouse_pos_changed && self.steer_cooldown >= STEER_COOLDOWN {
            self.mouse_pos_changed = false;
            self.steer_cooldown = Milliseconds(0);
            let mouse_angle = angle(self.mouse.pos);
            let send_angle = (251.0 * mouse_angle / (2.0 * PI)).floor() as u8;
            if self.last_send_angle != send_angle {                
                self.last_send_angle = send_angle;
                connection.send(vec![send_angle]);
            }
        }
    }

    /// Handles snake boosting.
    fn handle_snake_boost(&mut self, delta: Milliseconds, connection: &mut dyn Connection) {
        const BOOST_COOLDOWN: Milliseconds = Milliseconds(150);

        self.boost_cooldown += delta;
        if self.mouse_pressed_changed && self.boost_cooldown >= BOOST_COOLDOWN {
            self.mouse_pressed_changed = false;
            self.boost_cooldown = Milliseconds(0);
            if self.mouse.pressed {
                connection.send(START_BOOST_MESSAGE.to_vec());
            } else {
                connection.send(STOP_BOOST_MESSAGE.to_vec());
            }
        }
    }
}

/// Returns the angle, in radians, in the range `[0, 2π)`, from the x-axis to the given
/// point. The direction of the angle is determined by the rotation from the x-axis to
/// the y-axis.
fn angle(point: Point2) -> f32 {
    let (x, y) = (point.x, point.y);

    let angle = y.atan2(x);
    if angle >= 0.0 {
        angle
    } else {
        angle + 2.0 * PI
    }
}