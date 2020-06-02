use sir_eolith::State;
use sir_eolith::ggez_prelude::*;
use sir_eolith::consts::*;

fn main() -> GameResult<()> {
    let (ref mut ctx, events_loop) =
        &mut ContextBuilder::new("sir-eolith", "Nilirad")
        .window_setup(WindowSetup::default().title("sir-eolith"))
        .window_mode(WindowMode::default().dimensions(WINDOW_X, WINDOW_Y))
        .build()?;
    
    let state = &mut State;

    event::run(ctx, events_loop, state)
}
