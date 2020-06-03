use crate::consts::*;
use std::net::TcpStream;
use std::thread::{self, JoinHandle};
use crossbeam_channel::{unbounded, Sender, Receiver};
use websocket::{
    client::builder::ClientBuilder,
    header::Headers,
    header::Origin,
    receiver::Reader,
    sender::Writer,
    Message,
    OwnedMessage,
};

type Msg = Vec<u8>;

pub struct WebSocket {
    /// Sends data to server.
    sender: Writer<TcpStream>,
    /// Channel receiver that gets messages from the WebSocket listener thread.
    channel: Receiver<Msg>,
    /// Join handle for WebSocket listener thread.
    handle: JoinHandle<()>,
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

        let (tx, rx) = unbounded::<Msg>();
        
        let (mut receiver, sender) = client.split().unwrap(); // TODO: ugly.

        // A thread is spawned because the incoming_messages() method is thread-blocking.
        let handle = thread::spawn(move || {
            for message in receiver.incoming_messages() {
                if let Ok(message) = message {
                    match message {
                        OwnedMessage::Binary(data) => {
                            if let Err(error) = tx.send(data) {
                                println!("Error: {}", error);
                            }
                        }
                        OwnedMessage::Close(_) => {
                            break;
                        }
                        _ => (),
                    }
                }
            }
        });

        Self {
            sender,
            channel: rx,
            handle,
        }
    }

    pub fn poll_messages(&mut self) -> Vec<Msg> {
        self.channel.try_iter().collect()
    }

    pub fn send_message(&mut self, message: Msg) {
        if let Err(error) = self.sender.send_message(&OwnedMessage::Binary(message)) {
            println!("Error: {}", error);
        }
    }
}