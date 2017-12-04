use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use std::fmt::{Display, Formatter};
use std::fmt;
use std::error;

use listener::Listener;
use settings::Settings;
use loirc::{self, connect};
use loirc::{ActivityMonitor, Code, Event, Message, Prefix, Writer};

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

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::AlreadyClosed => write!(f, "Connection has already been closed"),
            Error::AlreadyDisconnected => write!(f, "Client has already been disconnected"),
            Error::Closed => write!(f, "Connection is closed"),
            Error::Disconnected => write!(f, "Client has been disconnected"),
            Error::IoError(ref err) => write!(f, "Client encountered I/O error: {}", err),
            Error::Multiline => write!(f, "Message contains line break")
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::AlreadyClosed => "Connection is already closed",
            Error::AlreadyDisconnected => "Connection is already disconnected",
            Error::Closed => "Connection has been manually closed",
            Error::Disconnected => "Connection has been dropped",
            Error::IoError(ref err) => err.description(),
            Error::Multiline => "Message contains a line break"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref err) => Some(err),
            _ => None
        }
    }
}

/// Ability to send commands to the irc server.
///
/// This trait represents the ability to send messages
/// to a server. It's implemented by Irc and Deferred.
pub trait IrcWrite {

    /// Send a raw message. A newline is added for you.
    ///
    /// If you add a new line it will be refused as a multiline message.
    fn raw<S: AsRef<str>>(&self, raw: S) -> Result<(), Error>;

    /// NICK command.
    fn nick(&self, nickname: &str) -> Result<(), Error> {
        self.raw(format!("NICK {}", nickname))
    }

    /// USER command.
    fn user(&self, username: &str, realname: &str) -> Result<(), Error> {
        self.raw(format!("USER {} 8 * :{}", username, realname))
    }

    /// PING command.
    fn ping(&self, server: &str) -> Result<(), Error> {
        self.raw(format!("PING {}", server))
    }

    /// PONG command.
    fn pong(&self, server: &str) -> Result<(), Error> {
        self.raw(format!("PONG {}", server))
    }

    /// PASS command.
    fn pass(&self, password: &str) -> Result<(), Error> {
        self.raw(format!("PASS {}", password))
    }

    /// PRIVMSG command.
    fn privmsg(&self, target: &str, text: &str) -> Result<(), Error> {
        self.raw(format!("PRIVMSG {} :{}", target, text))
    }

    /// NOTICE command.
    fn notice(&self, target: &str, text: &str) -> Result<(), Error> {
        self.raw(format!("NOTICE {} :{}", target, text))
    }

    /// JOIN command.
    fn join(&self, channel: &str, password: Option<&str>) -> Result<(), Error> {
        match password {
            None => self.raw(format!("JOIN {}", channel)),
            Some(password) => self.raw(format!("JOIN {} {}", channel, password)),
        }
    }

    /// PART command.
    fn part(&self, channel: &str, message: Option<&str>) -> Result<(), Error> {
        match message {
            None => self.raw(format!("PART {}", channel)),
            Some(message) => self.raw(format!("PART {} :{}", channel, message)),
        }
    }

    /// QUIT command.
    fn quit(&self, message: Option<&str>) -> Result<(), Error> {
        match message {
            None => self.raw(format!("QUIT :No message")),
            Some(message) => self.raw(format!("QUIT :{}", message)),
        }
    }

    /// Retrive the topic of a given channel.
    ///
    /// The `topic` event will receive the information.
    fn get_topic(&self, channel: &str) -> Result<(), Error> {
        self.raw(format!("TOPIC {}", channel))
    }

    /// Set the topic of a channel.
    ///
    /// To remove the topic of a channel, use an empty topic string.
    /// It will also trigger a `topic_change` event.
    fn set_topic(&self, channel: &str, topic: &str) -> Result<(), Error> {
        self.raw(format!("TOPIC {} :{}", channel, topic))
    }

    /// KICK command.
    fn kick(&self, channel: &str, nickname: &str) -> Result<(), Error> {
        self.raw(format!("KICK {} {}", channel, nickname))
    }

}

/// Status of a user inside a channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelUserStatus {
    /// User has special status.
    Normal,
    /// User has voice status.
    Voice,
    /// User has half operator status.
    HalfOperator,
    /// User has operator status.
    Operator,
    /// User has owner status.
    Owner,
}

/// User inside a channel.
///
/// Note that the same person might be in many channels. In any case, there will
/// be a ChannelUser object for each Channel the person is in.
#[derive(Debug)]
pub struct ChannelUser {
    /// Nickname of the user.
    nickname: Mutex<Arc<String>>,
    /// Status of the user inside the channel.
    status: Mutex<ChannelUserStatus>,
}

impl ChannelUser {

    fn new(nickname: &str, status: ChannelUserStatus) -> ChannelUser {
        ChannelUser {
            nickname: Mutex::new(Arc::new(nickname.into())),
            status: Mutex::new(status),
        }
    }

    fn from_raw(raw: &str) -> ChannelUser {
        let status = match raw.chars().next() {
            Some('~') => ChannelUserStatus::Owner,
            Some('&') => ChannelUserStatus::Owner,
            Some('%') => ChannelUserStatus::HalfOperator,
            Some('@') => ChannelUserStatus::Operator,
            Some('+') => ChannelUserStatus::Voice,
            _ => ChannelUserStatus::Normal,
        };

        let nickname = if status == ChannelUserStatus::Normal {
            raw
        } else {
            &raw[1..]
        };

        ChannelUser::new(nickname, status)
    }

    /// Get the nickname of the user.
    pub fn nickname(&self) -> Arc<String> {
        self.nickname.lock().unwrap().clone()
    }

    /// Get the status of the user.
    pub fn status(&self) -> ChannelUserStatus {
        *self.status.lock().unwrap()
    }

    fn set_nickname(&self, nickname: &str) {
        *self.nickname.lock().unwrap() = Arc::new(nickname.into());
    }

    fn set_status(&self, status: ChannelUserStatus) {
        *self.status.lock().unwrap() = status;
    }

}

/// Channel
#[derive(Debug)]
pub struct Channel {
    users: Mutex<Vec<Arc<ChannelUser>>>,
    /// Name of the channel.
    name: String,
    /// Topic of the channel.
    topic: Mutex<Option<Arc<String>>>,
}

impl Channel {

    fn new(name: &str) -> Channel {
        Channel {
            users: Mutex::new(Vec::new()),
            name: name.into(),
            topic: Mutex::new(None),
        }
    }

    /// Get the name of the channel.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the topic of the channel.
    pub fn topic(&self) -> Option<Arc<String>> {
        self.topic.lock().unwrap().clone()
    }

    /// Get a ChannelUser object from this channel using the user's nickname.
    pub fn user(&self, nickname: &str) -> Option<Arc<ChannelUser>> {
        let users = self.users.lock().unwrap();

        for user in users.iter() {
            if *user.nickname() == nickname {
                return Some(user.clone());
            }
        }

        None
    }

    /// Get the list of users in this channel.
    pub fn users(&self) -> Vec<Arc<ChannelUser>> {
        self.users.lock().unwrap().clone()
    }

    fn add_user(&self, user: Arc<ChannelUser>) {
        self.users.lock().unwrap().push(user);
    }

    fn remove_user(&self, nickname: &str) -> Option<Arc<ChannelUser>> {
        let mut users = self.users.lock().unwrap();

        if let Some(pos) = users.iter().position(|u| *u.nickname() == nickname) {
            Some(users.remove(pos))
        } else {
            None
        }
    }

    fn set_topic(&self, topic: &str) {
        *self.topic.lock().unwrap() = if topic.is_empty() {
            None
        } else {
            Some(Arc::new(topic.into()))
        };
    }

}

/// Status of the connection.
#[derive(Clone, Copy, Debug)]
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

/// Contains the connection to the server and the data about channels and users.
pub struct Irc {
    writer: Writer,
    channels: Mutex<HashMap<String, Arc<Channel>>>,
    status: Mutex<ConnectionStatus>,
}

impl Irc {

    /// Get a channel by name.
    pub fn channel(&self, name: &str) -> Option<Arc<Channel>> {
        self.get_channel_by_id(&name.to_lowercase())
    }

    /// Get the list of channels.
    pub fn channels(&self) -> Vec<Arc<Channel>> {
        self.channels.lock().unwrap().values().map(|v| v.clone()).collect::<Vec<Arc<Channel>>>()
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

    fn new(writer: Writer) -> Irc {
        Irc {
            writer: writer,
            status: Mutex::new(ConnectionStatus::Connected),
            channels: Mutex::new(HashMap::new()),
        }
    }

    fn get_channel_by_id(&self, id: &str) -> Option<Arc<Channel>> {
        self.channels.lock().unwrap().get(id).map(|c| c.clone())
    }

    fn ensure_channel_exists(&self, name: &str, id: &str) {
        self.channels.lock().unwrap().entry(id.into()).or_insert(Arc::new(Channel::new(name)));
    }

    fn channel_set_topic(&self, channel_id: &str, topic: &str) {
        let mut channels = self.channels.lock().unwrap();

        let channel = some_or_return!(channels.get_mut(channel_id));
        channel.set_topic(topic.into());
    }

    fn channel_add_user(&self, channel_id: &str, raw: &str) {
        let mut channels = self.channels.lock().unwrap();
        let channel = some_or_return!(channels.get_mut(channel_id));
        channel.add_user(Arc::new(ChannelUser::from_raw(raw)));
    }

    fn channel_del_user(&self, channel_id: &str, nickname: &str) -> Option<Arc<ChannelUser>> {
        if let Some(channel) = self.get_channel_by_id(channel_id) {
            channel.remove_user(nickname);
        }
        None
    }

    fn channel_update_user_mode(&self, channel_id: &str, nickname: &str, mode: &str) -> Option<(ChannelUserStatus, ChannelUserStatus)> {
        if let Some(channel) = self.get_channel_by_id(channel_id) {
            if let Some(user) = channel.user(nickname) {
                let old_status = user.status();

                match old_status {
                    ChannelUserStatus::Normal => {
                        match &mode[..] {
                            "+v" => user.set_status(ChannelUserStatus::Voice),
                            "+h" => user.set_status(ChannelUserStatus::HalfOperator),
                            "+o" => user.set_status(ChannelUserStatus::Operator),
                            _ => (),
                        }
                    }
                    ChannelUserStatus::HalfOperator => {
                        match &mode[..] {
                            "-h" => user.set_status(ChannelUserStatus::Normal),
                            _ => (),
                        }
                    },
                    ChannelUserStatus::Voice => {
                        match &mode[..] {
                            "-v" => user.set_status(ChannelUserStatus::Normal),
                            _ => (),
                        }
                    }
                    ChannelUserStatus::Operator | ChannelUserStatus::Owner => {
                        match &mode[..] {
                            "-o" => user.set_status(ChannelUserStatus::Normal),
                            _ => (),
                        }
                    }
                }

                return Some((old_status, user.status()));
            }
        }
        None
    }

    fn clear_channels(&self) {
        self.channels.lock().unwrap().clear();
    }

    fn set_status(&self, status: ConnectionStatus) {
        *self.status.lock().unwrap() = status;
    }

}

impl IrcWrite for Irc {

    fn raw<S: AsRef<str>>(&self, raw: S) -> Result<(), Error> {
        let raw = raw.as_ref();
        if raw.contains("\n") || raw.contains("\r") {
            return Err(Error::Multiline)
        }
        try!(self.writer.raw(format!("{}\n", raw)));
        Ok(())
    }

}

/// Create an irc client with the listener and settings.
pub fn dispatch<L: Listener>(listener: L, settings: Settings) -> Result<(), Error> {
    let (writer, reader) = try!(connect(settings.addr, settings.reconnection, settings.encoding));

    let irc = Irc::new(writer.clone());
    if !settings.password.is_empty() {
        try!(irc.pass(settings.password));
    }
    try!(irc.nick(settings.nickname));
    try!(irc.user(settings.username, settings.realname));

    let mut dispatch = Dispatch {
        am: settings.monitor.map(|s| ActivityMonitor::new(&writer, s)),
        listener: Box::new(listener),
        irc: Arc::new(irc),
        settings: settings,
    };

    for event in reader.iter() {
        dispatch.feed(&event);
    }

    Ok(())
}

struct Dispatch<'a> {
    am: Option<ActivityMonitor>,
    listener: Box<Listener + 'a>,
    irc: Arc<Irc>,
    settings: Settings<'a>,
}

impl<'a> Dispatch<'a> {

    /// Feed an event to the dispatcher.
    pub fn feed(&mut self, event: &Event) {
        if let Some(am) = self.am.as_ref() {
            am.feed(event);
        }

        self.listener.any(self.irc.clone(), event);

        match *event {
            Event::Closed(reason) => {
                self.irc.set_status(ConnectionStatus::Closed(reason));
                self.listener.close(self.irc.clone(), reason);
            }
            Event::Disconnected => {
                self.irc.set_status(ConnectionStatus::Disconnected);
                self.irc.clear_channels();
                self.listener.disconnect(self.irc.clone());
            }
            Event::Reconnecting => {
                self.irc.set_status(ConnectionStatus::Reconnecting);
                self.listener.reconnecting(self.irc.clone());
            }
            Event::Reconnected => {
                self.irc.set_status(ConnectionStatus::Connected);
                if self.settings.auto_ident {
                    let _ = self.irc.user(self.settings.username, self.settings.realname);
                    let _ = self.irc.nick(self.settings.nickname);
                }
                self.listener.reconnect(self.irc.clone());
            }
            Event::Message(ref msg) => {
                self.listener.msg(self.irc.clone(), &msg);
                if msg.code.is_error() {
                    self.listener.error_msg(self.irc.clone(), &msg.code, &msg);
                }
                match msg.code {
                    Code::RplWelcome => {
                        self.listener.welcome(self.irc.clone());
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
                        self.message(msg, false);
                    }
                    Code::Notice => {
                        self.message(msg, true);
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
                    Code::Mode => {
                        self.mode(msg);
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
        let channel = some_or_return!(self.irc.channel(&channel_name));
        self.listener.channel_join(self.irc.clone(), channel);
    }

    fn topic(&mut self, msg: &Message) {
        let topic = some_or_return!(msg.args.last());
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        self.irc.ensure_channel_exists(&channel_id, channel_name);
        self.irc.channel_set_topic(&channel_id, topic);

        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        self.listener.topic_change(self.irc.clone(), channel.clone(), channel.topic());
    }

    fn rpl_topic(&mut self, msg: &Message) {
        let topic = some_or_return!(msg.args.last());
        let channel_name = some_or_return!(msg.args.get(1));
        let channel_id = channel_name.to_lowercase();

        self.irc.ensure_channel_exists(&channel_id, channel_name);
        self.irc.channel_set_topic(&channel_id, topic);

        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        self.listener.topic(self.irc.clone(), channel.clone(), channel.topic());
    }

    fn rpl_no_topic(&mut self, msg: &Message) {
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        self.irc.ensure_channel_exists(channel_name, &channel_id);
        self.irc.channel_set_topic(&channel_id, "");

        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        self.listener.topic(self.irc.clone(), channel, None);
    }

    fn join(&mut self, msg: &Message) {
        let prefix = user_or_return!(msg.prefix);
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        self.irc.channel_add_user(&channel_id, &prefix.nickname);

        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        let user = some_or_return!(channel.user(&prefix.nickname));
        self.listener.user_join(self.irc.clone(), channel, user);
    }

    fn part(&mut self, msg: &Message) {
        let prefix = user_or_return!(msg.prefix);
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        self.irc.channel_del_user(&channel_id, &prefix.nickname);

        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        let user = some_or_return!(channel.user(&prefix.nickname));
        self.listener.user_part(self.irc.clone(), channel, user);
    }

    fn message(&mut self, msg: &Message, notice: bool) {
        let prefix = user_or_return!(msg.prefix);
        let text = some_or_return!(msg.args.last());
        let source = some_or_return!(msg.args.get(0));

        if source.starts_with("#") {
            let channel = some_or_return!(self.irc.channel(&source));
            let user = some_or_return!(channel.user(&prefix.nickname));
            if !notice {
                self.listener.channel_msg(self.irc.clone(), channel, user, text);
            } else {
                self.listener.channel_notice(self.irc.clone(), channel, user, text);
            }
        } else {
            if !notice {
                self.listener.private_msg(self.irc.clone(), prefix, text);
            } else {
                self.listener.private_notice(self.irc.clone(), prefix, text);
            }
        }
    }

    fn quit(&mut self, msg: &Message) {
        let user = user_or_return!(msg.prefix);

        for channel in self.irc.channels() {
            channel.remove_user(&user.nickname);
        }

        self.listener.user_quit(self.irc.clone(), &user.nickname);
    }

    fn nick(&mut self, msg: &Message) {
        let prefix = user_or_return!(msg.prefix);
        let newname = some_or_return!(msg.args.last());

        for channel in self.irc.channels() {
            if let Some(user) = channel.user(&prefix.nickname) {
                user.set_nickname(newname);
            }
        }

        self.listener.nick_change(self.irc.clone(), &prefix.nickname, &newname);
    }

    fn kick(&mut self, msg: &Message) {
        let kicked_user = some_or_return!(msg.args.last());
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        let channel_user = some_or_return!(self.irc.channel_del_user(&channel_id, kicked_user));
        let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
        self.listener.kick(self.irc.clone(), channel, channel_user);
    }

    fn ping(&mut self, msg: &Message) {
        let server = some_or_return!(msg.args.last());
        if self.settings.auto_ping {
            let _ = self.irc.pong(server);
        }
        self.listener.ping(self.irc.clone(), server);
    }

    fn pong(&mut self, msg: &Message) {
        let server = some_or_return!(msg.args.last());
        self.listener.pong(self.irc.clone(), server);
    }

    fn mode(&mut self, msg: &Message) {
        let mode = some_or_return!(msg.args.get(1));
        let nickname = some_or_return!(msg.args.get(2));
        let channel_name = some_or_return!(msg.args.get(0));
        let channel_id = channel_name.to_lowercase();

        if let Some((old_status, new_status)) = self.irc.channel_update_user_mode(&channel_id, nickname, mode) {
            if old_status != new_status {
                let channel = some_or_return!(self.irc.get_channel_by_id(&channel_id));
                let user = some_or_return!(channel.user(nickname));
                let status = user.status();
                self.listener.user_mode_change(self.irc.clone(), channel, user, old_status, status);
            }
        }
    }

}

#[test]
fn test_user_from_raw_norm() {
    let user = ChannelUser::from_raw("TEST");
    assert_eq!(&*user.nickname(), "TEST");
    assert_eq!(user.status(), ChannelUserStatus::Normal);
}

#[test]
fn test_user_from_raw_voice() {
    let user = ChannelUser::from_raw("+TEst");
    assert_eq!(&*user.nickname(), "TEst");
    assert_eq!(user.status(), ChannelUserStatus::Voice);
}

#[test]
fn test_user_from_raw_op() {
    let user = ChannelUser::from_raw("@test");
    assert_eq!(&*user.nickname(), "test");
    assert_eq!(user.status(), ChannelUserStatus::Operator);
}

#[test]
fn test_user_from_raw_owner() {
    let user = ChannelUser::from_raw("&test");
    assert_eq!(&*user.nickname(), "test");
    assert_eq!(user.status(), ChannelUserStatus::Owner);
}

#[test]
fn test_channel() {
    let channel = Channel::new("#testchannel");
    channel.set_topic("ABC DEF");

    let usr1 = Arc::new(ChannelUser::new("abc1", ChannelUserStatus::Normal));
    let usr2 = Arc::new(ChannelUser::new("abc2", ChannelUserStatus::Operator));

    channel.add_user(usr1.clone());
    channel.add_user(usr2.clone());

    assert_eq!(channel.name(), "#testchannel");
    assert_eq!(channel.topic(), Some(Arc::new("ABC DEF".into())));
    assert_eq!(channel.user("abc1").unwrap().nickname(), usr1.nickname());
    assert_eq!(channel.user("abc2").unwrap().nickname(), usr2.nickname());
}
