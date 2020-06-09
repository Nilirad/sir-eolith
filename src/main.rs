#[macro_use]
extern crate log;

use sir_eolith::State;
use sir_eolith::ws::WebSocket;
use sir_eolith::ggez_prelude::*;
use sir_eolith::consts::*;
use sir_eolith::get_playback;

fn main() -> GameResult<()> {
    env_logger::init();
    info!("Logger initialized.");

    let mut playback = false;
    let args: std::env::Args = std::env::args();
    for arg in args.skip(1) {
        if arg == "playback" {
            playback = true;
        }
    }

    let (ref mut ctx, events_loop) =
        &mut ContextBuilder::new("sir-eolith", "Nilirad")
        .window_setup(WindowSetup::default().title("sir-eolith"))
        .window_mode(WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()?;

    if playback {
        let mut playback = get_playback("373.json");
        let ref mut state = State::new(&mut playback);
        event::run(ctx, events_loop, state)?;
    } else {
        let (mut ws, ws_handle) = WebSocket::new(SERVER_URL.to_owned());
        let state = &mut State::new(&mut ws);
        
        event::run(ctx, events_loop, state)?;

        match ws_handle.join() {
            Ok(_) => info!("WebSocket thread joined."),
            Err(error) => error!("Error joining websocket thread: {:?}", error),
        }
    }
    info!("Terminating application.");
    Ok(())
}