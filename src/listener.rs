use std::sync::Arc;

use loirc::Event;
use {Channel, ChannelUser, ChannelUserStatus, Code, Irc, Message, PrefixUser};

/// Implement this trait to handle events.
pub trait Listener {

    /// Any event.
    ///
    /// This includes everything sent by the irc server, i/o errors, disconnects, reconnects, etc.
    #[allow(unused_variables)]
    fn any(&mut self, irc: Arc<Irc>, event: &Event) {}

    /// Any message.
    ///
    /// This is not to be confused with `channel_msg` or `private_msg`!
    /// Messages are a subset of events, they're what the irc server sends.
    #[allow(unused_variables)]
    fn msg(&mut self, irc: Arc<Irc>, msg: &Message) {}

    /// Any error message.
    ///
    /// When the server sends an error message.
    #[allow(unused_variables)]
    fn error_msg(&mut self, irc: Arc<Irc>, code: &Code, err: &Message) {}

    /// When the connection is closed.
    ///
    /// It can happen if you manually close the connection, if you set the `ReconnectionSettings`
    /// to `DoNotReconnect` or if the maximun number of reconnection attempts is reached.
    #[allow(unused_variables)]
    fn close(&mut self, irc: Arc<Irc>, reason: &str) {}

    /// When the connection is broken.
    #[allow(unused_variables)]
    fn disconnect(&mut self, irc: Arc<Irc>) {}

    /// When an attempt to reconnect is made.
    #[allow(unused_variables)]
    fn reconnecting(&mut self, irc: Arc<Irc>) {}

    /// When the connection is re-established.
    #[allow(unused_variables)]
    fn reconnect(&mut self, irc: Arc<Irc>) {}

    /// When the server sends the welcome packet.
    #[allow(unused_variables)]
    fn welcome(&mut self, irc: Arc<Irc>) {}

    /// When the client sucessfully joins a channel.
    #[allow(unused_variables)]
    fn channel_join(&mut self, irc: Arc<Irc>, channel: Arc<Channel>) {}

    /// When a user joins a channel we are listening on.
    #[allow(unused_variables)]
    fn user_join(&mut self, irc: Arc<Irc>, channel: Arc<Channel>, user: Arc<ChannelUser>) {}

    /// When a user parts a channel we are listening on.
    #[allow(unused_variables)]
    fn user_part(&mut self, irc: Arc<Irc>, channel: Arc<Channel>, user: Arc<ChannelUser>) {}

    /// When a user quits.
    #[allow(unused_variables)]
    fn user_quit(&mut self, irc: Arc<Irc>, nickname: &str) {}

    /// When a channel message is received.
    #[allow(unused_variables)]
    fn channel_msg(&mut self, irc: Arc<Irc>, channel: Arc<Channel>, sender: Arc<ChannelUser>, message: &str) {}

    /// When a channel notice is received.
    #[allow(unused_variables)]
    fn channel_notice(&mut self, irc: Arc<Irc>, channel: Arc<Channel>, sender: Arc<ChannelUser>, message: &str) {}

    /// When a private message is received.
    #[allow(unused_variables)]
    fn private_msg(&mut self, irc: Arc<Irc>, sender: &PrefixUser, message: &str) {}

    /// When a private notice is received.
    #[allow(unused_variables)]
    fn private_notice(&mut self, irc: Arc<Irc>, sender: &PrefixUser, message: &str) {}

    /// Reply to a `get_topic` command and when joining a channel.
    ///
    /// Note that this event might be called before the channel's user list is populated.
    /// If a channel has no topic, this event will not be fired when you join a channel.
    /// It's safe to assume that a channel has no topic if this event is not fired when joining.
    #[allow(unused_variables)]
    fn topic(&mut self, irc: Arc<Irc>, channel: Arc<Channel>, topic: Option<Arc<String>>) {}

    /// When the topic of is changed by someone.
    ///
    /// If you use the `set_topic` method, you will get a `topic_change` event instead of a
    /// `topic` event.
    #[allow(unused_variables)]
    fn topic_change(&mut self, irc: Arc<Irc>, channel: Arc<Channel>, topic: Option<Arc<String>>) {}

    /// When the nick of a user changes.
    #[allow(unused_variables)]
    fn nick_change(&mut self, irc: Arc<Irc>, oldnick: &str, newnick: &str) {}

    /// When a user gets kicked from a channel.
    #[allow(unused_variables)]
    fn kick(&mut self, irc: Arc<Irc>, channel: Arc<Channel>, user: Arc<ChannelUser>) {}

    /// When the server sends a ping message.
    #[allow(unused_variables)]
    fn ping(&mut self, irc: Arc<Irc>, server: &str) {}

    /// When the server sends a pong message.
    #[allow(unused_variables)]
    fn pong(&mut self, irc: Arc<Irc>, server: &str) {}

    /// When the mode of a user in a channel changes.
    #[allow(unused_variables)]
    fn user_mode_change(&mut self, irc: Arc<Irc>, channel: Arc<Channel>, user: Arc<ChannelUser>,
                        old_status: ChannelUserStatus, new_status: ChannelUserStatus) {}
}
