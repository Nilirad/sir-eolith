use client::connection::{Connection, ConnectionState, ServerMsg};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::error::Error;
use std::time::Instant;
use serde::Deserialize;
use base64::decode;

#[derive(Deserialize, Debug)]
pub struct Frame {
    // UNIX timestamp of the recorded frame.
    time: f64,
    // The server message in Base64 format.
    data: String,
}

#[derive(Deserialize, Debug)]
pub struct Frames{
    messages: Vec<Frame>,
}

// TODO: The following two functions should be one, right?
pub fn frames(path: PathBuf) -> Result<Vec<Frame>, Box<dyn Error>> {
    let mut file = File::open(&path)?;
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)?;
    match serde_json::from_str::<Frames>(json_string.as_str()) {
        Ok(frames) => Ok(frames.messages),
        Err(error) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, error))),
    }
}

pub fn load_replay(path: PathBuf) -> Replay {
    Replay::new(frames(path).unwrap())
}

pub struct Replay {
    frames: Vec<Frame>,
    start_time: Instant,
    first_frame_time: f64,
}

impl Replay {
    pub fn new(frames: Vec<Frame>) -> Self {
        let first_frame_time = frames[0].time;
        Self {
            frames,
            start_time: Instant::now(),
            first_frame_time,
        }
    } 
}

impl Connection for Replay {
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

    fn state(&self) -> ConnectionState {
        ConnectionState::Unilateral
    }
}

// TODO: Determine where this should be. Probably you should have a playback session
// stored somewhere in this module.
/* #[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_frames() {
        let path = std::path::Path::new("res").join("373.json");
        let _frames = frames(path).unwrap();
    }
} */