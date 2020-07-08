use crate::consts::*;
use crate::ggez_prelude::*;
use crate::types::*;
use super::State;
use legion::prelude::*;

/// The area of the world visible to the player. The higher the number, the more zoomed
/// out the view will be.
const FIELD_OF_VIEW: f32 = 3.0;

impl<'connection> State<'connection> {
    /// Draws the gameplay elements.
    pub fn draw_game(&mut self, ctx: &mut Context) -> GameResult {
        self.update_viewport(ctx)?;

        let mut mesh_builder = MeshBuilder::new();

        self.draw_border(&mut mesh_builder);
        self.draw_food(&mut mesh_builder);
        self.draw_snakes(&mut mesh_builder)?;

        let mesh = mesh_builder.build(ctx)?;
        graphics::draw(ctx, &mesh, (na::Point2::new(0.0, 0.0),))?;

        Ok(())
    }

    #[allow(unused_assignments)]
    fn draw_border(&mut self, mut mesh_builder: &mut MeshBuilder) {
        mesh_builder = mesh_builder.circle(
            graphics::DrawMode::fill(),
            na::Point2::new(21600.0, 21600.0),
            21600.0 - 435.0,
            0.00001,
            Color::new(0.2, 0.2, 0.2, 1.0),
        );
    }

    /// Draws the snakes.
    fn draw_snakes(&mut self, mut mesh_builder: &mut MeshBuilder) -> GameResult {
        let query = <(Read<Id>, Read<Pos>, Read<Body>)>::query();
        for (id, pos, body) in query.iter(&mut self.world) {
            // TODO: Wrap in function `color_from_id()`
            let color = {
                let r = id.0 & 0xFF00;
                let g = id.0 & 0x00FF;
                let r = r as f32 / std::u16::MAX as f32;
                let g = g as f32 / std::u16::MAX as f32;
                ggez::graphics::Color::new(r, g, 1.0, 1.0)
            };
            let width = 29.0 * (1.0 + (body.points.len() - 2) as f32 / 106.0).min(6.0);

            let points = body.points.iter()
                .map(|v| ggez::mint::Point2 {x: v.x, y: v.y})
                .collect::<Vec<ggez::mint::Point2<f32>>>();

            mesh_builder = mesh_builder.line(
                points.as_slice(),
                width,
                color,
            )?;

            mesh_builder = mesh_builder.circle(
                graphics::DrawMode::fill(),
                na::Point::from(pos.0),
                width / 2.0,
                0.1,
                Color::new(0.0, 0.5, 0.0, 1.0),
            );
        }

        Ok(())
    }

    fn draw_food(&mut self, mut mesh_builder: &mut MeshBuilder) {
        const FOOD_COLOR: Color = Color{ r: 1.0, g: 1.0, b: 0.0, a: 1.0 };

        let query = <(Read<Pos>, Read<Size>)>::query()
            .filter(tag::<FoodTag>());
        for (pos, size) in query.iter(&mut self.world) {
            mesh_builder = mesh_builder.circle(
                graphics::DrawMode::fill(),
                na::Point2::new(pos.0.x, pos.0.y),
                *size,
                0.1,
                FOOD_COLOR,
            );
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