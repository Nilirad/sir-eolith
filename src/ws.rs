use crate::consts::*;
use crate::message::{ClientMsg, ServerMsg};
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
    /// Client is logging into the server.
    LoggingIn,
    /// Server created a player in the world. Client can now send gameplay input.
    Playing,
    /// Connection has been interrupted by client or server.
    Disconnected,
}

pub struct WebSocket {
    /// Sends data to server.
    sender: Writer<TcpStream>,
    channel: Receiver<ServerMsg>,
    /// Connection state.
    pub state: ConnectionState,
}

impl WebSocket {
    pub fn new(url: String) -> (Self, JoinHandle<()>) {
        let mut headers = Headers::new();
        headers.set(Origin(ORIGIN.to_owned()));

        // TODO: Return a Result from function to handle errors.
        let client = ClientBuilder::new(url.as_str())
            .unwrap() // TODO: ugly.
            .custom_headers(&headers)
            .connect_insecure()
            .unwrap(); // TODO: ugly.
        
        let (mut receiver, mut sender) = client.split().unwrap(); // TODO: ugly.

        let (tx, rx) = unbounded();

        let thread_handle = thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message = match message {
                    Ok(m) => m,
                    Err(e) => {
                        println!("Receive Loop: {:?}", e);
                        break;
                    }
                };
                match message {
                    OwnedMessage::Binary(m) => tx.send(ServerMsg::new(m)).unwrap(),
                    OwnedMessage::Close(_) => break,
                    _ => (),
                }
            }
            
        });

        Self::request_login(&mut sender);

        (
            Self {
                sender,
                channel: rx,
                state: ConnectionState::LoggingIn,
            },
            thread_handle,
        )
    }

    pub fn poll_messages(&mut self) -> Vec<ServerMsg> {
        self.channel.try_iter().collect()
    }

    pub fn send_message(&mut self, message: ClientMsg) {
        if let Err(error) = self.sender.send_message(&OwnedMessage::Binary(message)) {
            println!("Error: {}", error);
        }
    }

    pub fn close(&mut self) {
        if let Err(error) = self.sender.send_message(&OwnedMessage::Close(None)) {
            println!("Cannot close connection: {}", error);
        }
    }

    fn request_login(sender: &mut Writer<TcpStream>) {
        const LOGIN_REQUEST_MSG: [u8; 1] = [0x63];
        if let Err(error) = sender.send_message(&OwnedMessage::Binary(LOGIN_REQUEST_MSG.to_vec())) {
            println!("Error: {}", error);
        }
    }
}