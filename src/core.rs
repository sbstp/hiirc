use std::collections::HashMap;
use std::io;
use std::slice::Iter;

use listener::Listener;
use loirc::{self, connect};
use loirc::{ActivityMonitor, Code, Event, Message, MonitorSettings, Prefix, ReconnectionSettings, Writer};

/// Represents the status of a user inside of a channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelUserStatus {
    /// User has special status.
    Normal,
    /// User has voice status.
    Voice,
    /// User has operator status.
    Operator,
}

/// Represents a user inside of a channel.
#[derive(Debug)]
pub struct ChannelUser {
    /// Nickname of the user.
    pub nickname: String,
    /// Status of the user inside the channel.
    pub status: ChannelUserStatus,
}

impl ChannelUser {

    fn from_raw(raw: &str) -> ChannelUser {
        let status = match raw.chars().next() {
            Some('@') => ChannelUserStatus::Operator,
            Some('+') => ChannelUserStatus::Voice,
            _ => ChannelUserStatus::Normal,
        };
        ChannelUser {
            nickname: (if status == ChannelUserStatus::Normal { raw } else { &raw[1..] }).into(),
            status: status,
        }
    }
}

/// Represents a channel.
#[derive(Debug)]
pub struct Channel {
    users: Vec<ChannelUser>,
    /// Name of the channel.
    pub name: String,
    /// Topic of the channel.
    pub topic: Option<String>,
}

impl Channel {

    /// Get a user by nickname from this channel.
    pub fn get_user(&self, nickname: &str) -> Option<&ChannelUser> {
        match self.users.iter().position(|u| u.nickname == nickname) {
            Some(idx) => Some(&self.users[idx]),
            None => None,
        }
    }

    /// Get an iterator that iterates over the channel's users.
    pub fn users(&self) -> Iter<ChannelUser> {
        self.users.iter()
    }

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
    channels: HashMap<String, Channel>,
    /// Status of the connection.
    pub status: ConnectionStatus,
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
            self.channels.insert(id.into(), Channel {
                name: name.into(),
                users: Vec::new(),
                topic: None,
            });
        }
    }

    fn channel_set_topic(&mut self, channel_id: &str, topic: &str) {
        let channel = some_or_return!(self.channels.get_mut(channel_id));
        channel.topic = if topic.len() == 0 {
            None
        } else {
            Some(topic.into())
        };
    }

    fn channel_add_user(&mut self, channel_id: &str, raw: &str) {
        let channel = some_or_return!(self.channels.get_mut(channel_id));
        channel.users.push(ChannelUser::from_raw(raw));
    }

    fn channel_del_user(&mut self, channel_id: &str, nickname: &str) -> Option<ChannelUser> {
        if let Some(channel) = self.channels.get_mut(channel_id) {
            if let Some(pos) = channel.users.iter().position(|u| u.nickname == nickname) {
                return Some(channel.users.remove(pos));
            }
        }
        None
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

    /// Retrive the topic of a given channel.
    ///
    /// The `topic` event will receive the information.
    pub fn get_topic(&self, channel: &str) -> Result<(), Error> {
        self.raw(format!("TOPIC {}", channel))
    }

    /// Set the topic of a channel.
    ///
    /// To remove the topic of a channel, use an empty topic string.
    /// It will also trigger a `topic_change` event.
    pub fn set_topic(&self, channel: &str, topic: &str) -> Result<(), Error> {
        self.raw(format!("TOPIC {} :{}", channel, topic))
    }

}

/// Create an irc client with the listener and settings.
pub fn dispatch<L: Listener>(listener: L, settings: Settings) -> Result<(), Error> {
    let reco_settings = settings.reco_settings.unwrap_or(ReconnectionSettings::DoNotReconnect);
    let (writer, reader) = try!(connect(settings.addr, reco_settings));

    let irc = Irc::new(writer.clone());
    try!(irc.nick(settings.nickname));
    try!(irc.user(settings.username, settings.realname));

    let mut dispatch = Dispatch {
        am: settings.mon_settings.map(|s| ActivityMonitor::new(&writer, s)),
        listener: Box::new(listener),
        irc: irc,
    };

    for event in reader.iter() {
        dispatch.feed(&event);
    }

    Ok(())
}

struct Dispatch<'a> {
    am: Option<ActivityMonitor>,
    listener: Box<Listener + 'a>,
    irc: Irc,
}

impl<'a> Dispatch<'a> {

    /// Feed an event to the dispatcher.
    pub fn feed(&mut self, event: &Event) {
        if let Some(am) = self.am.as_ref() {
            am.feed(event);
        }

        self.listener.any(&self.irc, event);

        match *event {
            Event::Closed(reason) => {
                self.irc.status = ConnectionStatus::Closed(reason);
                self.listener.close(&self.irc, reason);
            }
            Event::Disconnected => {
                self.irc.status = ConnectionStatus::Disconnected;
                self.irc.channels.clear();
                self.listener.disconnect(&self.irc);
            }
            Event::Reconnecting => {
                self.irc.status = ConnectionStatus::Reconnecting;
                self.listener.reconnecting(&self.irc);
            }
            Event::Reconnected => {
                self.irc.status = ConnectionStatus::Connected;
                self.listener.reconnect(&self.irc);
            }
            Event::Message(ref msg) => {
                match msg.code {
                    Code::RplWelcome => {
                        self.listener.welcome(&self.irc);
                    }
                    Code::RplNamreply => {
                        self.name_reply(msg);
                    }
                    Code::RplEndofnames => {
                        self.end_name_reply(msg);
                    }
                    Code::Topic => {
                        self.topic(msg);
                    }
                    Code::RplTopic => {
                        self.rpl_topic(msg);
                    }
                    Code::RplNotopic => {
                        self.rpl_no_topic(msg);
                    }
                    Code::Join => {
                        self.join(msg);
                    }
                    Code::Part => {
                        self.part(msg);
                    }
                    Code::Privmsg => {
                        self.privmsg(msg);
                    }
                    Code::Quit => {
                        self.quit(msg);
                    }
                    Code::Nick => {
                        self.nick(msg);
                    }
                    Code::Kick => {
                        self.kick(msg);
                    }
                    Code::Ping => {
                        self.ping(msg);
                    }
                    Code::Pong => {
                        self.pong(msg);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn name_reply(&mut self, msg: &Message) {
        let channel_name = some_or_return!(msg.args.get(2));
        let channel_id = channel_name.to_lowercase();
        let user_list = some_or_return!(msg.args.last());

        self.irc.ensure_channel_exists(channel_name, &channel_id);
        for raw in user_list.split(" ") {
            self.irc.channel_add_user(&channel_id, raw);
        }
    }

    fn end_name_reply(&mut self, msg: &Message) {
        let channel_name = some_or_return!(msg.args.get(1));
        let channel = some_or_return!(self.irc.get_channel_by_name(&channel_name));
        self.listener.channel_join(&self.irc, channel);
    }

    fn topic(&mut self, msg: &Message) {
        let topic = some_or_return!(msg.args.last());
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        self.irc.ensure_channel_exists(&channel_id, channel_name);
        self.irc.channel_set_topic(&channel_id, topic);

        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        self.listener.topic_change(&self.irc, channel, channel.topic.as_ref().map(|t| &t[..]));
    }

    fn rpl_topic(&mut self, msg: &Message) {
        let topic = some_or_return!(msg.args.last());
        let channel_name = some_or_return!(msg.args.get(1));
        let channel_id = channel_name.to_lowercase();

        self.irc.ensure_channel_exists(&channel_id, channel_name);
        self.irc.channel_set_topic(&channel_id, topic);

        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        self.listener.topic(&self.irc, channel, channel.topic.as_ref().map(|t| &t[..]));
    }

    fn rpl_no_topic(&mut self, msg: &Message) {
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        self.irc.ensure_channel_exists(channel_name, &channel_id);
        self.irc.channel_set_topic(&channel_id, "");

        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        self.listener.topic(&self.irc, channel, channel.topic.as_ref().map(|t| &t[..]));
    }

    fn join(&mut self, msg: &Message) {
        let prefix = user_or_return!(msg.prefix);
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        self.irc.channel_add_user(&channel_id, &prefix.nickname);

        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        let user = some_or_return!(channel.get_user(&prefix.nickname));
        self.listener.user_join(&self.irc, channel, user);
    }

    fn part(&mut self, msg: &Message) {
        let prefix = user_or_return!(msg.prefix);
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        self.irc.channel_del_user(&channel_id, &prefix.nickname);

        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        let user = some_or_return!(channel.get_user(&prefix.nickname));
        self.listener.user_part(&self.irc, channel, user);
    }

    fn privmsg(&mut self, msg: &Message) {
        let prefix = user_or_return!(msg.prefix);
        let text = some_or_return!(msg.args.last());
        let source = some_or_return!(msg.args.get(0));

        if source.starts_with("#") {
            let channel = some_or_return!(self.irc.get_channel_by_name(&source));
            let user = some_or_return!(channel.get_user(&prefix.nickname));
            self.listener.channel_msg(&self.irc, channel, user, text);
        } else {
            self.listener.private_msg(&self.irc, &prefix.nickname, text);
        }
    }

    fn quit(&mut self, msg: &Message) {
        let user = user_or_return!(msg.prefix);

        for (_, channel) in self.irc.channels.iter_mut() {
            if let Some(pos) = channel.users.iter().position(|u| u.nickname == user.nickname) {
                channel.users.remove(pos);
            }
        }

        self.listener.user_quit(&self.irc, &user.nickname);
    }

    fn nick(&mut self, msg: &Message) {
        let prefix = user_or_return!(msg.prefix);
        let newname = some_or_return!(msg.args.last());

        for (_, channel) in self.irc.channels.iter_mut() {
            for user in channel.users.iter_mut() {
                if user.nickname == prefix.nickname {
                    user.nickname = newname.clone();
                }
            }
        }

        self.listener.nick_change(&self.irc, &prefix.nickname, &newname);
    }

    fn kick(&mut self, msg: &Message) {
        let kicked_user = some_or_return!(msg.args.last());
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        let channel_user = some_or_return!(self.irc.channel_del_user(&channel_id, kicked_user));
        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        self.listener.kick(&self.irc, &channel, &channel_user);
    }

    fn ping(&mut self, msg: &Message) {
        let server = some_or_return!(msg.args.last());
        let _ = self.irc.pong(server);
        self.listener.ping(&self.irc, server);
    }

    fn pong(&mut self, msg: &Message) {
        let server = some_or_return!(msg.args.last());
        self.listener.pong(&self.irc, server);
    }

}

#[test]
fn test_user_from_raw_norm() {
    let user = ChannelUser::from_raw("TEST");
    assert_eq!(&user.nickname, "TEST");
    assert_eq!(user.status, ChannelUserStatus::Normal);
}

#[test]
fn test_user_from_raw_voice() {
    let user = ChannelUser::from_raw("+TEst");
    assert_eq!(&user.nickname, "TEst");
    assert_eq!(user.status, ChannelUserStatus::Voice);
}

#[test]
fn test_user_from_raw_op() {
    let user = ChannelUser::from_raw("@test");
    assert_eq!(&user.nickname, "test");
    assert_eq!(user.status, ChannelUserStatus::Operator);
}
