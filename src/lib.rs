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
                        self.listener.channel_join(&mut self.state, &msg.args[0]);
                    }
                    Code::Join => {
                        self.join(msg);
                    }
                    Code::Part => {
                        self.part(msg);
                    }
                    Code::Quit => {
                        self.quit(msg);
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

    fn join(&mut self, msg: &Message) {
        if msg.args.len() >= 1 {
            if let Some(ref prefix) = msg.prefix {
                match *prefix {
                    Prefix::User(ref user) => {
                        let channel_name = &msg.args[0];
                        let channel_id = channel_name.to_lowercase();
                        if let Some(channel) = self.state.channels.get_mut(&channel_id) {
                            channel.users.push(user.nickname.to_owned());
                        }
                        if self.state.channels.contains_key(&channel_id) {
                            self.listener.user_join(&mut self.state, channel_name, &user.nickname);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn part(&mut self, msg: &Message) {
        if msg.args.len() >= 1 {
            if let Some(ref prefix) = msg.prefix {
                match *prefix {
                    Prefix::User(ref user) => {
                        let channel_name = &msg.args[0];
                        let channel_id = channel_name.to_lowercase();
                        if let Some(channel) = self.state.channels.get_mut(&channel_id) {
                            if let Some(pos) = channel.users.iter().position(|u| u == &user.nickname) {
                                channel.users.remove(pos);
                            }
                        }
                        self.listener.user_part(&mut self.state, channel_name, &user.nickname);
                    }
                    _ => {}
                }
            }
        }
    }

    fn quit(&mut self, msg: &Message) {
        if let Some(ref prefix) = msg.prefix {
            match *prefix {
                Prefix::User(ref user) => {
                    for (_, channel) in self.state.channels.iter_mut() {
                        if let Some(pos) = channel.users.iter().position(|u| u == &user.nickname) {
                            channel.users.remove(pos);
                        }
                    }
                    self.listener.user_quit(&mut self.state, &user.nickname);
                }
                _ => {}
            }
        }
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
    fn channel_join(&mut self, state: &mut State, channel: &str) {}

    /// When a user joins a channel we are listening on.
    #[allow(unused_variables)]
    fn user_join(&mut self, state: &mut State, channel: &str, nickname :&str) {}

    /// When a user parts a channel we are listening on.
    #[allow(unused_variables)]
    fn user_part(&mut self, state: &mut State, channel: &str, nickname :&str) {}

    /// When a user quits.
    #[allow(unused_variables)]
    fn user_quit(&mut self, state: &mut State, nickname: &str) {}

}
