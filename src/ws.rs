use crate::consts::*;
use crate::message::{ClientMsg, ServerMsg, Connection, ConnectionState};
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

pub struct WebSocket {
    /// Sends data to server.
    sender: Writer<TcpStream>,
    /// Receives messages from the Websocket listener thread.
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

        let thread_handle = thread::spawn(move || { // TODO: See if you can simplify.
            for message in receiver.incoming_messages() {
                let message = match message {
                    Ok(m) => m,
                    Err(e) => {
                        error!("[ws receiver] {:?}", e);
                        break;
                    }
                };
                match message {
                    OwnedMessage::Binary(m) => {
                        if let Err(error) = tx.send(ServerMsg::new(m)) {
                            error!("[ws receiver] Cannot send message to main thread: {}", error);
                        }
                    }
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

    fn request_login(sender: &mut Writer<TcpStream>) {
        const LOGIN_REQUEST_MSG: [u8; 1] = [0x63];
        if let Err(error) = sender.send_message(&OwnedMessage::Binary(LOGIN_REQUEST_MSG.to_vec())) {
            error!("Error: {}", error);
        }
    }
}

impl Connection for WebSocket {
    fn poll_messages(&mut self) -> Vec<ServerMsg> {
        self.channel.try_iter().collect()
    }

    fn send(&mut self, message: ClientMsg) {
        if let Err(error) = self.sender.send_message(&OwnedMessage::Binary(message)) {
            error!("Sending error: {}", error);
        }
    }

    fn close(&mut self) {
        if self.state != ConnectionState::Disconnected {
            if let Err(error) = self.sender.send_message(&OwnedMessage::Close(None)) {
                error!("Cannot close connection: {}", error);
            }
            self.state = ConnectionState::Disconnected;
        }
    }

    fn state(&self) -> ConnectionState {
        self.state
    }

    fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }
}