use crate::consts::*;
use crate::ggez_prelude::*;
use crate::types::*;
use super::State;
use legion::prelude::*;

impl<'connection> State<'connection> {
    /// Draws the gameplay elements.
    pub fn draw_game(&mut self, ctx: &mut Context) -> GameResult {
        self.player_coords(ctx)?;
        let mut mesh_builder = MeshBuilder::new();
        self.draw_border(&mut mesh_builder);
        self.draw_food(&mut mesh_builder);
        self.draw_preys(&mut mesh_builder);
        self.draw_snakes(&mut mesh_builder, ctx)?;
        let mesh = mesh_builder.build(ctx)?;
        graphics::draw(ctx, &mesh, (na::Point2::new(0.0, 0.0),))?;
        if self.cache.nametags_shown { self.draw_nicknames(ctx)?; }
        
        self.screen_coords(ctx)?;
        self.draw_minimap(ctx)?;
        self.draw_leaderboard(ctx);
        self.draw_score(ctx);
        self.draw_rank(ctx);
        self.draw_debug_info(ctx);
        graphics::draw_queued_text(
            ctx,
            graphics::DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )
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
    fn draw_snakes(&mut self, mut mesh_builder: &mut MeshBuilder, ctx: &mut Context) -> GameResult {
        let query = <(Read<Id>, Read<Pos>, Read<Body>)>::query();
        for (id, pos, body) in query.iter(&mut self.world) {
            // TODO: Wrap in function `color_from_id()`
            let color = {
                let r = *id & 0xFF00;
                let g = *id & 0x00FF;
                let r = r as f32 / std::u16::MAX as f32;
                let g = g as f32 / std::u16::MAX as f32;
                ggez::graphics::Color::new(r, g, 1.0, 1.0)
            };
            let width = 29.0 * body.sc();

            let mut points = body.points.iter()
                .map(|v| ggez::mint::Point2 {x: v.x, y: v.y})
                .collect::<Vec<ggez::mint::Point2<f32>>>();

            points.push(ggez::mint::Point2 {x: pos.x, y: pos.y});

            mesh_builder = mesh_builder.line(
                points.as_slice(),
                width,
                color,
            )?;

            mesh_builder = mesh_builder.circle(
                graphics::DrawMode::fill(),
                na::Point::from(*pos),
                width / 2.0,
                0.1,
                Color::new(0.0, 0.5, 0.0, 1.0),
            );

            if self.cache.nametags_shown {
                if let Some(nickname) = self.cache.nicknames.get(&id) {
                    graphics::queue_text(
                        ctx,
                        nickname,
                        na::Point::from(*pos),
                        Some(graphics::BLACK),
                    );
                }
            }

            /* let ang_line = vec![
                ggez::mint::Point2 {x: pos.x, y: pos.y},
                ggez::mint::Point2 {
                    x: pos.x + 100.0 * movement.angle.cos(),
                    y: pos.y + 100.0 * movement.angle.sin(),
                },
            ];

            let wang_line = vec![
                ggez::mint::Point2 {x: pos.x, y: pos.y},
                ggez::mint::Point2 {
                    x: pos.x + 100.0 * movement.target_angle.cos(),
                    y: pos.y + 100.0 * movement.target_angle.sin(),
                },
            ];

            let color = match movement.direction {
                Direction::None => Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
                Direction::Left => Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
                Direction::Right => Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
            };

            mesh_builder = mesh_builder.line(
                wang_line.as_slice(),
                5.0,
                Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            )?;

            mesh_builder = mesh_builder.line(
                ang_line.as_slice(),
                5.0,
                color,
            )?; */
        }

        Ok(())
    }

    fn draw_nicknames(&mut self, ctx: &mut Context) -> GameResult {
        graphics::draw_queued_text(
            ctx,
            (na::Point2::new(0.0, 0.0),),
            None,
            graphics::FilterMode::Linear,
        )
    }

    fn draw_food(&mut self, mut mesh_builder: &mut MeshBuilder) {
        const FOOD_COLOR: Color = Color{ r: 1.0, g: 1.0, b: 0.0, a: 1.0 };

        let query = <(Read<Pos>, Read<Size>)>::query()
            .filter(tag::<FoodTag>());
        for (pos, size) in query.iter(&mut self.world) {
            mesh_builder = mesh_builder.circle(
                graphics::DrawMode::fill(),
                na::Point2::new(pos.x, pos.y),
                *size,
                0.1,
                FOOD_COLOR,
            );
        }
    }

    fn draw_preys(&mut self, mut mesh_builder: &mut MeshBuilder) {
        const PREY_COLOR: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        
        let query = <(Read<Pos>, Read<Size>)>::query()
            .filter(tag::<PreyTag>());
        for (pos, size) in query.iter(&self.world) {
            mesh_builder = mesh_builder.circle(
                graphics::DrawMode::fill(),
                na::Point2::new(pos.x, pos.y),
                *size * 3.0, // TODO: Fix real size.
                0.5,
                PREY_COLOR,
            );

            /* let ang_line = vec![
                ggez::mint::Point2 {x: pos.x, y: pos.y},
                ggez::mint::Point2 {
                    x: pos.x + 100.0 * movement.angle.cos(),
                    y: pos.y + 100.0 * movement.angle.sin(),
                },
            ];

            let wang_line = vec![
                ggez::mint::Point2 {x: pos.x, y: pos.y},
                ggez::mint::Point2 {
                    x: pos.x + 100.0 * movement.target_angle.cos(),
                    y: pos.y + 100.0 * movement.target_angle.sin(),
                },
            ];

            let color = match movement.direction {
                Direction::None => Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
                Direction::Left => Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
                Direction::Right => Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
            };

            mesh_builder = mesh_builder.line(
                wang_line.as_slice(),
                5.0,
                Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            ).unwrap();

            mesh_builder = mesh_builder.line(
                ang_line.as_slice(),
                5.0,
                color,
            ).unwrap(); */
        }
    }

    fn draw_leaderboard(&mut self, ctx: &mut Context) {
        const OFFSET: f32 = 20.0;
        for (i, (rank, nickname, score)) in self.stats.leaderboard.iter().enumerate() {
            let y = 20.0 + i as f32 * OFFSET;
            graphics::queue_text(ctx, rank, na::Point2::new(20.0, y), None);
            graphics::queue_text(ctx, nickname, na::Point2::new(60.0, y), None);
            graphics::queue_text(ctx, score, na::Point2::new(360.0, y), None);
        }
    }

    fn draw_score(&mut self, ctx: &mut Context) {
        graphics::queue_text(
            ctx,
            &self.stats.score_text,
            na::Point2::new(20.0, WINDOW_HEIGHT - 50.0),
            None,
        );
    }

    fn draw_rank(&mut self, ctx: &mut Context) {
        graphics::queue_text(
            ctx,
            &self.stats.rank_text,
            na::Point2::new(20.0, WINDOW_HEIGHT - 30.0),
            None,
        );
    }

    fn draw_debug_info(&mut self, ctx: &mut Context) {
        const TEXT_X: f32 = WINDOW_WIDTH as f32 - MINIMAP_SIDE as f32 - 20.0 - 50.0;
        const TEXT_Y: f32 = WINDOW_HEIGHT as f32 - MINIMAP_SIDE as f32 - 100.0;
        
        self.debug_info.fps(ggez::timer::fps(ctx));
        self.debug_info.ping(self.ping_status.last_ping);
        
        graphics::queue_text(
            ctx,
            &self.debug_info.x,
            na::Point2::new(TEXT_X, TEXT_Y),
            None,
        );

        graphics::queue_text(
            ctx,
            &self.debug_info.y,
            na::Point2::new(TEXT_X + 60.0, TEXT_Y),
            None,
        );

        graphics::queue_text(
            ctx,
            &self.debug_info.fps,
            na::Point2::new(TEXT_X, TEXT_Y + 20.0),
            None,
        );

        graphics::queue_text(
            ctx,
            &self.debug_info.ping,
            na::Point2::new(TEXT_X, TEXT_Y + 40.0),
            None,
        );
    }

    fn draw_minimap(&mut self, ctx: &mut Context) -> GameResult {
        let mut minimap_mesh_builder = MeshBuilder::new();

        const MINIMAP_RADIUS: f32 = MINIMAP_SIDE as f32 / 2.0;
        minimap_mesh_builder.circle(
            graphics::DrawMode::fill(),
            na::Point2::new(MINIMAP_RADIUS, MINIMAP_RADIUS),
            MINIMAP_RADIUS + 3.0,
            0.01,
            graphics::BLACK,
        );
        minimap_mesh_builder.circle(
            graphics::DrawMode::fill(),
            na::Point2::new(MINIMAP_RADIUS, MINIMAP_RADIUS),
            MINIMAP_RADIUS,
            0.01,
            Color::new(0.4, 0.4, 0.4, 1.0),
        );

        for (i, flag) in self.stats.minimap_data.iter().enumerate() {
            if flag {
                let x = (i % MINIMAP_SIDE) as f32;
                let y = (i / MINIMAP_SIDE) as f32;

                let pixel_rect = Rect::new(x, y, 1.0, 1.0);
                minimap_mesh_builder.rectangle(
                    graphics::DrawMode::fill(),
                    pixel_rect,
                    graphics::WHITE,
                );
            }
        }

        let query = <(Read<Pos>,)>::query()
            .filter(tag_value(&PlayerTag(PlayerType::You)));
        for (pos,) in query.iter(&self.world) {
            let point = na::Point2::new(
                pos.x * MINIMAP_SIDE as f32 / (2.0 * self.params.world_radius),
                pos.y * MINIMAP_SIDE as f32 / (2.0 * self.params.world_radius),
            );
            minimap_mesh_builder.circle(
                graphics::DrawMode::fill(),
                point,
                4.0,
                1.0,
                graphics::BLACK,
            );
            minimap_mesh_builder.circle(
                graphics::DrawMode::fill(),
                point,
                2.0,
                1.0,
                Color::new(1.0, 1.0, 1.0, 1.0),
            );
        }

        if let Ok(ref minimap) = minimap_mesh_builder.build(ctx) {
            graphics::draw(
                ctx,
                minimap,
                (
                    na::Point2::new(
                        WINDOW_WIDTH - MINIMAP_SIDE as f32 - 20.0,
                        WINDOW_HEIGHT - MINIMAP_SIDE as f32 - 20.0,
                    ),
                ),
            )?;
        }

        Ok(())
    }
    
    /// Updates the viewport, centering it to the player's snake's head.
    fn player_coords(&mut self, ctx: &mut Context) -> GameResult {
        let query = <(Read<Pos>, Read<Body>)>::query()
            .filter(tag_value(&PlayerTag(PlayerType::You)));
        for (pos, body) in query.iter(&mut self.world) {
            self.cache.player_pos = *pos;
            self.cache.player_body_length = body.length();
            self.debug_info.pos(*pos);
        }

        let scale = 1.0 / self.zoom.factor(self.cache.player_body_length);
        let viewport = graphics::Rect {
            x: self.cache.player_pos.x - (scale * WINDOW_WIDTH / 2.0),
            y: self.cache.player_pos.y - (scale * WINDOW_HEIGHT / 2.0),
            w: scale * WINDOW_WIDTH,
            h: scale * WINDOW_HEIGHT,
        };
        graphics::set_screen_coordinates(ctx, viewport)?;

        Ok(())
    }

    fn screen_coords(&mut self, ctx: &mut Context) -> GameResult {
        let viewport = graphics::Rect {
            x: 0.0,
            y: 0.0,
            w: WINDOW_WIDTH,
            h: WINDOW_HEIGHT,
        };

        graphics::set_screen_coordinates(ctx, viewport)
    }
}