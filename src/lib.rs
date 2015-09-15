//! TODO
#![deny(missing_docs)]

extern crate loirc;

use std::collections::HashMap;
use std::io;

pub use loirc::{connect, ActivityMonitor, Code, Event, Message, MonitorSettings, Prefix, Reader, ReconnectionSettings, Writer};

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

/// Errors that can occur.
#[derive(Debug)]
pub enum Error {
    /// Connection is already closed.
    AlreadyClosed,
    /// Connection is already disconnected.
    AlreadyDisconnected,
    /// Connection was manually closed.
    Closed,
    /// Connection was dropped.
    ///
    /// A reconnection might be in process.
    Disconnected,
    /// I/O error.
    IoError(io::Error),
    /// The message contains a line break.
    Multiline,
}

impl From<loirc::Error> for Error {
    fn from(err: loirc::Error) -> Error {
        match err {
            loirc::Error::AlreadyClosed => Error::AlreadyClosed,
            loirc::Error::AlreadyDisconnected => Error::AlreadyDisconnected,
            loirc::Error::Closed => Error::Closed,
            loirc::Error::Disconnected => Error::Disconnected,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

/// Settings for the dispatcher.
pub struct Settings<'a> {
    /// Address of the irc server.
    pub addr: &'a str,
    /// Preferred nickname.
    pub nickname: &'a str,
    /// Username.
    pub username: &'a str,
    /// Real name.
    pub realname: &'a str,
    /// Reconnection settings. If None, reconnection is disabled.
    pub reco_settings: Option<ReconnectionSettings>,
    /// Monitor settings. If None, monitoring is disabled.
    pub mon_settings: Option<MonitorSettings>,
}

/// Represents the state of this connection.
pub struct Irc {
    writer: Writer,
    /// Status of the connection.
    pub status: ConnectionStatus,
    channels: HashMap<String, Channel>,
}

impl Irc {

    fn new(writer: Writer) -> Irc {
        Irc {
            writer: writer,
            status: ConnectionStatus::Connected,
            channels: HashMap::new(),
        }
    }

    /// Get a channel by name.
    pub fn get_channel(&self, chan: &str) -> Option<&Channel> {
        self.channels.get(&chan.to_lowercase())
    }

    /// Check if the underlying connection is closed.
    pub fn is_closed(&self) -> bool {
        self.writer.is_closed()
    }

    /// Close the underlying connection.
    pub fn close(&self) -> Result<(), Error> {
        try!(self.writer.close());
        Ok(())
    }

    /// Send a raw message. A newline is added for you.
    ///
    /// If you add a new line it will be refused as a multiline message.
    pub fn raw<S: AsRef<str>>(&self, raw: S) -> Result<(), Error> {
        let raw = raw.as_ref();
        if raw.contains("\n") || raw.contains("\r") {
            return Err(Error::Multiline)
        }
        try!(self.writer.raw(format!("{}\n", raw)));
        Ok(())
    }

    /// NICK command.
    pub fn nick(&self, nickname: &str) -> Result<(), Error> {
        self.raw(format!("NICK {}", nickname))
    }

    /// USER command.
    pub fn user(&self, username: &str, realname: &str) -> Result<(), Error> {
        self.raw(format!("USER {} 8 * :{}", username, realname))
    }

    /// PING command.
    pub fn ping(&self, server: &str) -> Result<(), Error> {
        self.raw(format!("PING {}", server))
    }

    /// PONG command.
    pub fn pong(&self, server: &str) -> Result<(), Error> {
        self.raw(format!("PONG {}", server))
    }

    /// PRIVMSG command.
    pub fn privmsg(&self, target: &str, text: &str) -> Result<(), Error> {
        self.raw(format!("PRIVMSG {} :{}", target, text))
    }

    /// JOIN command.
    pub fn join(&self, channel: &str, password: Option<&str>) -> Result<(), Error> {
        match password {
            None => self.raw(format!("JOIN {}", channel)),
            Some(password) => self.raw(format!("JOIN {} {}", channel, password)),
        }
    }

    /// PART command.
    pub fn part(&self, channel: &str, message: Option<&str>) -> Result<(), Error> {
        match message {
            None => self.raw(format!("PART {}", channel)),
            Some(message) => self.raw(format!("PART {} :{}", channel, message)),
        }
    }

    /// QUIT command.
    pub fn quit(&self, message: Option<&str>) -> Result<(), Error> {
        match message {
            None => self.raw(format!("QUIT :No message")),
            Some(message) => self.raw(format!("QUIT :{}", message)),
        }
    }

}

/// Create an irc client with the listener and settings.
pub fn dispatch<L: Listener>(listener: L, settings: Settings) -> Result<(), Error> {
    let mut listener = Box::new(listener);
    let reco_settings = settings.reco_settings.unwrap_or(ReconnectionSettings::DoNotReconnect);
    let (writer, reader) = try!(connect(settings.addr, reco_settings));

    let mut irc = Irc::new(writer.clone());
    try!(irc.nick(settings.nickname));
    try!(irc.user(settings.username, settings.realname));

    if let Some(mon_settings) = settings.mon_settings {
        let am = ActivityMonitor::new(&writer, mon_settings);
        for event in reader.iter() {
            am.feed(&event);
            feed(&mut listener, &mut irc, &event);
        }
    } else {
        for event in reader.iter() {
            println!("{:?}", &event);
            feed(&mut listener, &mut irc, &event);
        }
    }

    Ok(())
}

/// Feed an event to the dispatcher.
fn feed<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, event: &Event) {
    listener.any(irc, event);

    match *event {
        Event::Closed(reason) => {
            irc.status = ConnectionStatus::Closed(reason);
        }
        Event::Disconnected => {
            irc.status = ConnectionStatus::Disconnected;
            irc.channels.clear();
        }
        Event::Reconnecting => {
            irc.status = ConnectionStatus::Reconnecting;
        }
        Event::Message(ref msg) => {
            match msg.code {
                Code::RplWelcome => {
                    listener.welcome(irc);
                }
                Code::RplNamreply => {
                    name_reply(listener, irc, msg);
                }
                Code::RplEndofnames => {
                    listener.channel_join(irc, &msg.args[1]);
                }
                Code::Join => {
                    join(listener, irc, msg);
                }
                Code::Part => {
                    part(listener, irc, msg);
                }
                Code::Quit => {
                    quit(listener, irc, msg);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn name_reply<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    let channel_name = &msg.args[2];
    let channel_id = channel_name.to_lowercase();

    if !irc.channels.contains_key(&channel_id) {
        irc.channels.insert(channel_id.clone(), Channel {
            name: channel_name.to_owned(),
            users: Vec::new(),
        });
    }

    if let Some(ref suffix) = msg.suffix {
        let channel = irc.channels.get_mut(&channel_id).unwrap();
        for nick in suffix.split(" ") {
            channel.users.push(nick.to_owned());
        }
    }
}

fn join<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    if msg.args.len() >= 1 {
        if let Some(ref prefix) = msg.prefix {
            match *prefix {
                Prefix::User(ref user) => {
                    let channel_name = &msg.args[0];
                    let channel_id = channel_name.to_lowercase();
                    if let Some(channel) = irc.channels.get_mut(&channel_id) {
                        channel.users.push(user.nickname.to_owned());
                    }
                    if irc.channels.contains_key(&channel_id) {
                        listener.user_join(irc, channel_name, &user.nickname);
                    }
                }
                _ => {}
            }
        }
    }
}

fn part<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    if msg.args.len() >= 1 {
        if let Some(ref prefix) = msg.prefix {
            match *prefix {
                Prefix::User(ref user) => {
                    let channel_name = &msg.args[0];
                    let channel_id = channel_name.to_lowercase();
                    if let Some(channel) = irc.channels.get_mut(&channel_id) {
                        if let Some(pos) = channel.users.iter().position(|u| u == &user.nickname) {
                            channel.users.remove(pos);
                        }
                    }
                    listener.user_part(irc, channel_name, &user.nickname);
                }
                _ => {}
            }
        }
    }
}

fn quit<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    if let Some(ref prefix) = msg.prefix {
        match *prefix {
            Prefix::User(ref user) => {
                for (_, channel) in irc.channels.iter_mut() {
                    if let Some(pos) = channel.users.iter().position(|u| u == &user.nickname) {
                        channel.users.remove(pos);
                    }
                }
                listener.user_quit(irc, &user.nickname);
            }
            _ => {}
        }
    }
}

/// Implement this trait to handle events.
pub trait Listener {

    /// Any event.
    #[allow(unused_variables)]
    fn any(&mut self, irc: &mut Irc, event: &Event) {}

    /// When the server sends the welcome packet.
    #[allow(unused_variables)]
    fn welcome(&mut self, irc: &mut Irc) {}

    /// When the client sucessfully joins a channel.
    #[allow(unused_variables)]
    fn channel_join(&mut self, irc: &mut Irc, channel: &str) {}

    /// When a user joins a channel we are listening on.
    #[allow(unused_variables)]
    fn user_join(&mut self, irc: &mut Irc, channel: &str, nickname :&str) {}

    /// When a user parts a channel we are listening on.
    #[allow(unused_variables)]
    fn user_part(&mut self, irc: &mut Irc, channel: &str, nickname :&str) {}

    /// When a user quits.
    #[allow(unused_variables)]
    fn user_quit(&mut self, irc: &mut Irc, nickname: &str) {}

    /// When a private message is received.
    #[allow(unused_variables)]
    fn privmsg(&mut self, irc: &mut Irc, message: &str) {}

}
