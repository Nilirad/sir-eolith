use crate::consts::*;
use crate::Msg;
use std::collections::VecDeque;
use std::net::TcpStream;
use std::thread::{self, JoinHandle};
use crossbeam_channel::{unbounded, Receiver};
use websocket::{
    client::builder::ClientBuilder,
    header::Headers,
    header::Origin,
    sender::Writer,
    OwnedMessage,
};

/// Tracks connection state between client and server.
#[derive(Debug, Eq, PartialEq)]
pub enum ConnectionState {
    /// Client didn't request login.
    Uninitialized,
    /// Client requested login, server may not have responded yet.
    SentLoginRequest,
    /// Client sent idba, server may not have responded yet.
    SentIdba,
    /// Login successful. Gameplay started.
    Playing,
    /// Client or server have closed the connection.
    Disconnected,
}

pub struct WebSocket {
    /// Sends data to server.
    sender: Writer<TcpStream>,
    channel: Receiver<Msg>,
    handle: JoinHandle<()>,
    /// Connection state.
    pub state: ConnectionState,
}

impl WebSocket {
    pub fn new(url: String) -> Self {
        let mut headers = Headers::new();
        headers.set(Origin(ORIGIN.to_owned()));

        println!("Setting up WebSocket...");

        // TODO: Return a Result from function to handle errors.
        let client = ClientBuilder::new(url.as_str())
            .unwrap() // TODO: ugly.
            .custom_headers(&headers)
            .connect_insecure()
            .unwrap(); // TODO: ugly.
        
        let (mut receiver, sender) = client.split().unwrap(); // TODO: ugly.

        let (tx, rx) = unbounded();

        let handle = thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message = match message {
                    Ok(m) => m,
                    Err(e) => {
                        println!("Receive Loop: {:?}", e);
                        break;
                    }
                };
                match message {
                    OwnedMessage::Binary(m) => tx.send(m).unwrap(),
                    OwnedMessage::Close(_) => break,
                    _ => (),
                }
            }
            
        });

        println!("Done.");

        Self {
            sender,
            channel: rx,
            handle,
            state: ConnectionState::Uninitialized,
        }
    }

    pub fn poll_messages(&mut self) -> VecDeque<Msg> {
        self.channel.try_iter().collect()
    }

    pub fn send_message(&mut self, message: Msg) {
        if let Err(error) = self.sender.send_message(&OwnedMessage::Binary(message)) {
            println!("Error: {}", error);
        }
    }
}