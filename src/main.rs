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

    let (ref mut ctx, events_loop) =
        &mut ContextBuilder::new("sir-eolith", "Nilirad")
        .window_setup(WindowSetup::default().title("sir-eolith"))
        .window_mode(WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()?;

    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => {
            let (mut ws, ws_handle) = WebSocket::new(SERVER_URL.to_owned());
            let state = &mut State::new(&mut ws);
            event::run(ctx, events_loop, state)?;
            match ws_handle.join() {
                Ok(_) => info!("WebSocket thread joined."),
                Err(error) => error!("Error joining websocket thread: {:?}", error),
            }
        }
        2 => {
            let mut playback = get_playback(args[1].as_str());
            let ref mut state = State::new(&mut playback);
            event::run(ctx, events_loop, state)?;
        }
        _ => (), // TODO: show help.
    }

    info!("Terminating application.");
    Ok(())
}