use crate::consts::*;
use crate::ggez_prelude::*;
use crate::components::*;
use super::State;
use legion::prelude::*;

/// The area of the world visible to the player. The higher the number, the more zoomed
/// out the view will be.
const FIELD_OF_VIEW: f32 = 3.0;

impl State {
    /// Draws the gameplay elements.
    pub fn draw_game(&mut self, ctx: &mut Context) -> GameResult {        
        self.update_viewport(ctx)?;
        self.draw_snakes(ctx)?;

        Ok(())
    }

    /// Draws the snakes.
    fn draw_snakes(&mut self, ctx: &mut Context) -> GameResult {
        let query = <(Read<Id>, Read<SnakeSegments>)>::query();
        for (id, segments) in query.iter(&mut self.world) {
            // TODO: Wrap in function `color_from_id()`
            let color = {
                let r = id.0 & 0xFF00;
                let g = id.0 & 0x00FF;
                let r = r as f32 / std::u16::MAX as f32;
                let g = g as f32 / std::u16::MAX as f32;
                ggez::graphics::Color::new(r, g, 1.0, 1.0)
            };
            for section in segments.iter() {
                let screen_x = section.0.x;
                let screen_y = section.0.y;
                let circle = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    na::Point2::new(screen_x, screen_y),
                    30.0,
                    2.0,
                    color,
                )?;
                graphics::draw(
                    ctx,
                    &circle,
                    (na::Point2::new(0.0, 0.0),)
                )?;
            }
        }
        Ok(())
    }
    
    /// Updates the viewport, centering it to the player's snake's head.
    fn update_viewport(&mut self, ctx: &mut Context) -> GameResult {
        let query = <(Read<Pos>,)>::query()
            .filter(tag_value(&PlayerTag(PlayerType::You)));
        
        for (pos,) in query.iter(&mut self.world) {
            let viewport = graphics::Rect {
                x: pos.0.x - (FIELD_OF_VIEW * WINDOW_WIDTH / 2.0),
                y: pos.0.y - (FIELD_OF_VIEW * WINDOW_HEIGHT / 2.0),
                w: FIELD_OF_VIEW * WINDOW_WIDTH,
                h: FIELD_OF_VIEW * WINDOW_HEIGHT,
            };
            graphics::set_screen_coordinates(ctx, viewport)?;
        }
        Ok(())
    }
}