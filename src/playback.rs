use crate::message::{Connection, ConnectionState, ClientMsg, ServerMsg};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;
use std::time::Instant;
use serde::Deserialize;
use base64::decode;

#[derive(Deserialize, Debug)]
pub struct PlaybackFrame {
    // UNIX timestamp of the recorded frame.
    time: f64,
    // The server message in Base64 format.
    data: String,
}

#[derive(Deserialize, Debug)]
pub struct Frames{
    messages: Vec<PlaybackFrame>,
}

pub fn frames() -> Result<Frames, Box<dyn Error>> {
    let  path = Path::new("res/373.json");
    let mut file = File::open(&path)?;
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)?;
    match serde_json::from_str::<Frames>(json_string.as_str()) {
        Ok(frames) => Ok(frames),
        Err(error) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, error))),
    }
}

pub struct PlaybackSession {
    frames: Vec<PlaybackFrame>,
    start_time: Instant,
    first_frame_time: f64,
}

impl PlaybackSession {
    pub fn new(frames: Vec<PlaybackFrame>) -> Self {
        let first_frame_time = frames[0].time;
        Self {
            frames,
            start_time: Instant::now(),
            first_frame_time,
        }
    } 
}

impl Connection for PlaybackSession {
    fn poll_messages(&mut self) -> Vec<ServerMsg> {
        let elapsed_time = (Instant::now() - self.start_time).as_secs_f64();

        let mut drain_index = 0usize;
        for frame in self.frames.iter() {
            if elapsed_time >= frame.time - self.first_frame_time {
                drain_index += 1;
            } else {
                break; // frames are ordered by ascending time.
            }
        }

        // TODO: That unwrap(). Can we trust Chrome Dev Tools?
        self.frames.drain(..drain_index)
            .map(|f| ServerMsg::new(decode(f.data).unwrap()))
            .collect()
    }

    /// This connection type does not forward any input.
    fn send(&mut self, _message: ClientMsg) {}

    fn close(&mut self) {
        todo!();
    }

    fn state(&self) -> ConnectionState {
        unimplemented!();
    }

    fn set_state(&mut self, state: ConnectionState) {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_frames() {
        let _frames = frames().unwrap();
        
    }
}