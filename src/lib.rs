//! TODO
#![deny(missing_docs)]

extern crate loirc;

#[macro_use]
mod macros;
mod listener;

use std::collections::HashMap;
use std::io;

pub use loirc::{connect, ActivityMonitor, Code, Duration, Event, Message, MonitorSettings, Prefix, Reader, ReconnectionSettings, User, Writer};

pub use listener::Listener;

/// Represents a channel.
#[derive(Debug)]
pub struct Channel {
    /// Name of the channel.
    pub name: String,
    /// List of users by nickname.
    pub users: Vec<String>,
    /// Topic of the channel.
    pub topic: Option<String>,
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
    pub fn get_channel_by_name(&self, name: &str) -> Option<&Channel> {
        self.get_channel_by_id(&name.to_lowercase())
    }

    /// Get a channel by id.
    pub fn get_channel_by_id(&self, id: &str) -> Option<&Channel> {
        self.channels.get(id)
    }

    // Ensure a channel if it does not exist.
    fn ensure_channel_exists(&mut self, name: &str, id: &str) {
        if !self.channels.contains_key(id) {
            self.channels.insert(id.to_owned(), Channel {
                name: name.to_owned(),
                users: Vec::new(),
                topic: None,
            });
        }
    }

    fn set_channel_topic(&mut self, channel_id: &str, topic: &str) {
        let channel = some_or_return!(self.channels.get_mut(channel_id));
        channel.topic = if topic.len() == 0 {
            None
        } else {
            Some(topic.to_owned())
        };
    }

    fn add_user(&mut self, channel_id: &str, nickname: &str) {
        let channel = some_or_return!(self.channels.get_mut(channel_id));
        channel.users.push(nickname.to_owned());
    }

    fn del_user(&mut self, channel_id: &str, nickname: &str) {
        let channel = some_or_return!(self.channels.get_mut(channel_id));
        if let Some(pos) = channel.users.iter().position(|u| u == nickname) {
            channel.users.remove(pos);
        }
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
    pub fn nick<S: AsRef<str>>(&self, nickname: S) -> Result<(), Error> {
        self.raw(format!("NICK {}", nickname.as_ref()))
    }

    /// USER command.
    pub fn user<S: AsRef<str>>(&self, username: S, realname: S) -> Result<(), Error> {
        self.raw(format!("USER {} 8 * :{}", username.as_ref(), realname.as_ref()))
    }

    /// PING command.
    pub fn ping<S: AsRef<str>>(&self, server: S) -> Result<(), Error> {
        self.raw(format!("PING {}", server.as_ref()))
    }

    /// PONG command.
    pub fn pong<S: AsRef<str>>(&self, server: S) -> Result<(), Error> {
        self.raw(format!("PONG {}", server.as_ref()))
    }

    /// PRIVMSG command.
    pub fn privmsg<S: AsRef<str>>(&self, target: S, text: S) -> Result<(), Error> {
        self.raw(format!("PRIVMSG {} :{}", target.as_ref(), text.as_ref()))
    }

    /// JOIN command.
    pub fn join<S: AsRef<str>>(&self, channel: S, password: Option<S>) -> Result<(), Error> {
        match password {
            None => self.raw(format!("JOIN {}", channel.as_ref())),
            Some(password) => self.raw(format!("JOIN {} {}", channel.as_ref(), password.as_ref())),
        }
    }

    /// PART command.
    pub fn part<S: AsRef<str>>(&self, channel: S, message: Option<S>) -> Result<(), Error> {
        match message {
            None => self.raw(format!("PART {}", channel.as_ref())),
            Some(message) => self.raw(format!("PART {} :{}", channel.as_ref(), message.as_ref())),
        }
    }

    /// QUIT command.
    pub fn quit<S: AsRef<str>>(&self, message: Option<S>) -> Result<(), Error> {
        match message {
            None => self.raw(format!("QUIT :No message")),
            Some(message) => self.raw(format!("QUIT :{}", message.as_ref())),
        }
    }

    /// Retrive the topic of a given channel. The topic event will receive the information.
    pub fn get_topic<S: AsRef<str>>(&self, channel: S) -> Result<(), Error> {
        self.raw(format!("TOPIC {}", channel.as_ref()))
    }

    /// Set the topic of a channel.
    ///
    /// To remove the topic of a channel, use an empty topic string.
    /// It will also trigger a topic change event.
    pub fn set_topic<C: AsRef<str>, T: AsRef<str>>(&self, channel: C, topic: T) -> Result<(), Error> {
        self.raw(format!("TOPIC {} :{}", channel.as_ref(), topic.as_ref()))
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
                    name_reply(irc, msg);
                }
                Code::RplEndofnames => {
                    end_name_reply(listener, irc, msg);
                }
                Code::Topic => {
                    topic(listener, irc, msg);
                }
                Code::RplTopic => {
                    rpl_topic(listener, irc, msg);
                }
                Code::RplNotopic => {
                    rpl_no_topic(listener, irc, msg);
                }
                Code::Join => {
                    join(listener, irc, msg);
                }
                Code::Part => {
                    part(listener, irc, msg);
                }
                Code::Privmsg => {
                    privmsg(listener, irc, msg);
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

fn name_reply(irc: &mut Irc, msg: &Message) {
    let channel_name = some_or_return!(msg.args.get(2));
    let channel_id = channel_name.to_lowercase();
    let user_list = some_or_return!(msg.suffix.as_ref());

    irc.ensure_channel_exists(channel_name, &channel_id);
    let channel = some_or_return!(irc.channels.get_mut(&channel_id));
    for nick in user_list.split(" ") {
        channel.users.push(nick.to_owned());
    }
}

fn end_name_reply<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    let channel_name = some_or_return!(msg.args.get(1));
    listener.channel_join(irc, channel_name);
}

fn topic<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    let topic = some_or_return!(msg.suffix.as_ref());
    let channel_name = some_or_return!(msg.args.get(0));
    let channel_id = channel_name.to_lowercase();

    irc.ensure_channel_exists(&channel_id, channel_name);
    irc.set_channel_topic(&channel_id, topic);

    let channel = some_or_return!(irc.get_channel_by_id(&channel_id));
    listener.topic_change(&irc, channel, channel.topic.as_ref().map(|t| &t[..]));
}

fn rpl_topic<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    let topic = some_or_return!(msg.suffix.as_ref());
    let channel_name = some_or_return!(msg.args.get(1));
    let channel_id = channel_name.to_lowercase();

    irc.ensure_channel_exists(&channel_id, channel_name);
    irc.set_channel_topic(&channel_id, topic);

    let channel = some_or_return!(irc.get_channel_by_id(&channel_id));
    listener.topic(&irc, channel, channel.topic.as_ref().map(|t| &t[..]));
}

fn rpl_no_topic<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    let channel_name = some_or_return!(msg.args.get(0));
    let channel_id = channel_name.to_lowercase();

    irc.ensure_channel_exists(channel_name, &channel_id);
    irc.set_channel_topic(&channel_id, "");

    let channel = some_or_return!(irc.get_channel_by_id(&channel_id));
    listener.topic(&irc, channel, channel.topic.as_ref().map(|t| &t[..]));
}

fn join<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    let user = user_or_return!(msg.prefix);
    let channel_name = some_or_return!(msg.args.get(0));
    let channel_id = channel_name.to_lowercase();

    irc.add_user(&channel_id, &user.nickname);
    listener.user_join(irc, channel_name, &user.nickname);
}

fn part<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    let user = user_or_return!(msg.prefix);
    let channel_name = some_or_return!(msg.args.get(0));
    let channel_id = channel_name.to_lowercase();

    irc.del_user(&channel_id, &user.nickname);
    listener.user_part(irc, channel_name, &user.nickname)
}

fn privmsg<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    let user = user_or_return!(msg.prefix);
    let text = some_or_return!(msg.suffix.as_ref());
    let source = some_or_return!(msg.args.get(0));

    if source.starts_with("#") {
        listener.channel_msg(irc, &user.nickname, source, text);
    } else {
        listener.private_msg(irc, &user.nickname, text);
    }
}

fn quit<L: Listener>(listener: &mut Box<L>, irc: &mut Irc, msg: &Message) {
    let user = user_or_return!(msg.prefix);

    for (_, channel) in irc.channels.iter_mut() {
        if let Some(pos) = channel.users.iter().position(|u| u == &user.nickname) {
            channel.users.remove(pos);
        }
    }

    listener.user_quit(irc, &user.nickname);
}
