use crate::consts::*;
use std::net::TcpStream;
use websocket::{
    client::builder::ClientBuilder,
    header::Headers,
    header::Origin,
    receiver::Reader,
    sender::Writer,
};

pub struct WebSocket {
    pub receiver: Reader<TcpStream>,
    pub sender: Writer<TcpStream>,
}

impl WebSocket {
    pub fn new(url: String) -> Self {
        let mut headers = Headers::new();
        headers.set(Origin(ORIGIN.to_owned()));

        let client = ClientBuilder::new(url.as_str())
            .unwrap() // TODO: ugly.
            .custom_headers(&headers)
            .connect_insecure()
            .unwrap();
        
        let (receiver, sender) = client.split().unwrap(); // TODO: ugly.

        Self {
            receiver,
            sender,
        }
    }
}