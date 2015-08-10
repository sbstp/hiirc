//! TODO
#![deny(missing_docs)]

extern crate loirc;

use std::collections::HashMap;

pub use loirc::*;

/// Represents a channel.
#[derive(Debug)]
pub struct Channel {
    /// Name of the channel.
    pub name: String,
    /// List of users by nickname.
    pub users: Vec<String>,
}

/// Status of the connection.
#[derive(Debug)]
pub enum ConnectionStatus {
    /// Connection was closed.
    Closed(&'static str),
    /// Connection is alive.
    Connected,
    /// Connection was dropped.
    Disconnected,
    /// Attempting to reconnect.
    Reconnecting,
}

/// Represents the state of this connection.
#[derive(Debug)]
pub struct State {
    /// Status of the connection.
    pub status: ConnectionStatus,
    channels: HashMap<String, Channel>,
}

impl State {

    fn new() -> State {
        State {
            status: ConnectionStatus::Connected,
            channels: HashMap::new(),
        }
    }

    /// Get a channel by name.
    pub fn get_channel(&self, chan: &str) -> Option<&Channel> {
        self.channels.get(&chan.to_lowercase())
    }

}

/// Dispatches the raw events to functions.
pub struct Dispatcher {
    state: State,
    listener: Box<Listener + 'static>,
}

impl Dispatcher {

    /// Create a new dispatcher from the given listener.
    pub fn new<L: Listener + 'static>(listener: L) -> Dispatcher {
        Dispatcher {
            state: State::new(),
            listener: Box::new(listener),
        }
    }

    /// Borrow the state of check the data.
    pub fn borrow_state(&mut self) -> &State {
        &self.state
    }

    /// Feed an event to the dispatcher.
    pub fn feed(&mut self, event: &Event) {
        self.listener.any(&mut self.state, event);

        match *event {
            Event::Closed(reason) => {
                self.state.status = ConnectionStatus::Closed(reason);
            }
            Event::Disconnected => {
                self.state.status = ConnectionStatus::Disconnected;
                self.state.channels.clear();
            }
            Event::Reconnecting => {
                self.state.status = ConnectionStatus::Reconnecting;
            }
            Event::Message(ref msg) => {
                match msg.code {
                    Code::RplWelcome => {
                        self.listener.welcome(&mut self.state);
                    }
                    Code::RplNamreply => {
                        self.name_reply(msg);
                    }
                    Code::RplEndofnames => {
                        self.end_of_names(msg);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn name_reply(&mut self, msg: &Message) {
        let channel_name = &msg.args[2];
        let channel_id = channel_name.to_lowercase();

        if !self.state.channels.contains_key(&channel_id) {
            self.state.channels.insert(channel_id.clone(), Channel {
                name: channel_name.to_owned(),
                users: Vec::new(),
            });
        }

        if let Some(ref suffix) = msg.suffix {
            let channel = self.state.channels.get_mut(&channel_id).unwrap();
            for nick in suffix.split(" ") {
                channel.users.push(nick.to_owned());
            }
        }
    }

    fn end_of_names(&mut self, msg: &Message) {
        self.listener.channel_join(&mut self.state, &msg.args[0]);
    }

}

/// Implement this trait to handle events.
pub trait Listener {

    /// Any event.
    #[allow(unused_variables)]
    fn any(&mut self, state: &mut State, event: &Event) {}

    /// When the server sends the welcome packet.
    #[allow(unused_variables)]
    fn welcome(&mut self, state: &mut State) {}

    /// When the client sucessfully joins a channel.
    #[allow(unused_variables)]
    fn channel_join(&mut self, state: &mut State, name: &str) {}

}
