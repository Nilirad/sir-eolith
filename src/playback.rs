use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
pub struct PlaybackFrame {
    time: f64,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_frames() {
        let _frames = frames().unwrap();
        
    }
}