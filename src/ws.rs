use crate::consts::*;
use std::net::TcpStream;
use std::thread::{self, JoinHandle};
use crossbeam_channel::{unbounded, Sender, Receiver};
use websocket::{
    client::builder::ClientBuilder,
    header::Headers,
    header::Origin,
    receiver::Reader,
    result::{WebSocketResult, WebSocketError},
    sender::Writer,
    Message,
    OwnedMessage,
};

type Msg = Vec<u8>;

pub struct WebSocket {
    /// Reads data coming from server.
    receiver: Reader<TcpStream>,
    /// Sends data to server.
    sender: Writer<TcpStream>,
}

impl WebSocket {
    pub fn new(url: String) -> Self {
        let mut headers = Headers::new();
        headers.set(Origin(ORIGIN.to_owned()));

        let client = ClientBuilder::new(url.as_str())
            .unwrap() // TODO: ugly.
            .custom_headers(&headers)
            .connect_insecure()
            .unwrap(); // TODO: ugly.
        
        let (mut receiver, sender) = client.split().unwrap(); // TODO: ugly.

        Self {
            receiver,
            sender,
        }
    }

    pub fn poll_messages(&mut self) -> Vec<Msg> {
        let mut messages = Vec::new();
        while let Ok(message) = self.receiver.recv_message() {
            if let OwnedMessage::Binary(message_bytes) = message {
                messages.push(message_bytes);
            }
        }
        messages
    }

    pub fn send_message(&mut self, message: Msg) {
        if let Err(error) = self.sender.send_message(&OwnedMessage::Binary(message)) {
            println!("Error: {}", error);
        }
    }
}