use sir_eolith::State;
use sir_eolith::ws::WebSocket;
use sir_eolith::ggez_prelude::*;
use sir_eolith::consts::*;

fn main() -> GameResult<()> {
    env_logger::init();

    let (ref mut ctx, events_loop) =
        &mut ContextBuilder::new("sir-eolith", "Nilirad")
        .window_setup(WindowSetup::default().title("sir-eolith"))
        .window_mode(WindowMode::default().dimensions(WINDOW_X, WINDOW_Y))
        .build()?;
    
    let (ws, ws_handle) = WebSocket::new(SERVER_URL.to_owned());
    let state = &mut State::new(ws);

    event::run(ctx, events_loop, state)?;

    if let Err(error) = ws_handle.join() {
        println!("Error joining websocket thread: {:?}", error);
    }

    Ok(())
}
