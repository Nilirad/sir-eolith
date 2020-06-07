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

        let mut mesh_builder = MeshBuilder::new();
        let mut valid_builder = false;

        self.draw_food(&mut mesh_builder, &mut valid_builder);
        self.draw_snakes(&mut mesh_builder, &mut valid_builder);

        //debug!("{:?}", mesh_builder);
        if valid_builder {
            let mesh = mesh_builder.build(ctx)?;
            graphics::draw(ctx, &mesh, (na::Point2::new(0.0, 0.0),))?;
        }

        Ok(())
    }

    /// Draws the snakes.
    fn draw_snakes(&mut self, mut mesh_builder: &mut MeshBuilder, valid_builder: &mut bool) {
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
            for segment in segments.iter() {
                let screen_x = segment.0.x;
                let screen_y = segment.0.y;
                mesh_builder = mesh_builder.circle(
                    graphics::DrawMode::fill(),
                    na::Point2::new(screen_x, screen_y),
                    30.0,
                    2.0,
                    color,
                );
                *valid_builder = true;
            }
        }
    }

    fn draw_food(&mut self, mut mesh_builder: &mut MeshBuilder, valid_builder: &mut bool) {
        const FOOD_COLOR: Color = Color{ r: 1.0, g: 1.0, b: 0.0, a: 1.0 };

        let query = <(Read<Pos>,)>::query()
            .filter(tag::<FoodTag>());
        for (pos,) in query.iter(&mut self.world) {
            mesh_builder = mesh_builder.circle(
                graphics::DrawMode::fill(),
                na::Point2::new(pos.0.x, pos.0.y),
                10.0,
                2.0,
                FOOD_COLOR,
            );
            *valid_builder = true;
        }
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