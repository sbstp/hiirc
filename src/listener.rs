use loirc::Event;

use {Channel, ChannelUser, Irc};

/// Implement this trait to handle events.
pub trait Listener {

    /// Any event.
    #[allow(unused_variables)]
    fn any(&mut self, irc: &Irc, event: &Event) {}

    /// When the connection is closed.
    ///
    /// It can happen if you manually close the connection, if you set the `ReconnectionSettings`
    /// to `DoNotReconnect` or if the maximun number of reconnection attempts is reached.
    #[allow(unused_variables)]
    fn close(&mut self, irc: &Irc, reason: &str) {}

    /// When the connection is broken.
    #[allow(unused_variables)]
    fn disconnect(&mut self, irc: &Irc) {}

    /// When an attempt to reconnect is made.
    #[allow(unused_variables)]
    fn reconnecting(&mut self, irc: &Irc) {}

    /// When the connection is re-established.
    #[allow(unused_variables)]
    fn reconnect(&mut self, irc: &Irc) {}

    /// When the server sends the welcome packet.
    #[allow(unused_variables)]
    fn welcome(&mut self, irc: &Irc) {}

    /// When the client sucessfully joins a channel.
    #[allow(unused_variables)]
    fn channel_join(&mut self, irc: &Irc, channel: &Channel) {}

    /// When a user joins a channel we are listening on.
    #[allow(unused_variables)]
    fn user_join(&mut self, irc: &Irc, channel: &Channel, user :&ChannelUser) {}

    /// When a user parts a channel we are listening on.
    #[allow(unused_variables)]
    fn user_part(&mut self, irc: &Irc, channel: &Channel, user :&ChannelUser) {}

    /// When a user quits.
    #[allow(unused_variables)]
    fn user_quit(&mut self, irc: &Irc, nickname: &str) {}

    /// When a channel message is received.
    #[allow(unused_variables)]
    fn channel_msg(&mut self, irc: &Irc, channel: &Channel, sender: &ChannelUser, message: &str) {}

    /// When a channel notice is received.
    #[allow(unused_variables)]
    fn channel_notice(&mut self, irc: &Irc, channel: &Channel, sender: &ChannelUser, message: &str) {}

    /// When a private message is received.
    #[allow(unused_variables)]
    fn private_msg(&mut self, irc: &Irc, sender: &str, message: &str) {}

    /// When a private notice is received.
    #[allow(unused_variables)]
    fn private_notice(&mut self, irc: &Irc, sender: &str, message: &str) {}

    /// Reply to a `get_topic` command and when joining a channel.
    ///
    /// Note that this event might be called before the channel's user list is populated.
    /// If a channel has no topic, this event will not be fired when you join a channel.
    /// It's safe to assume that a channel has no topic if this event is not fired when joining.
    #[allow(unused_variables)]
    fn topic(&mut self, irc: &Irc, channel: &Channel, topic: Option<&str>) {}

    /// When the topic of is changed by someone.
    ///
    /// If you use the `set_topic` method, you will get a `topic_change` event instead of a
    /// `topic` event.
    #[allow(unused_variables)]
    fn topic_change(&mut self, irc: &Irc, channel: &Channel, topic: Option<&str>) {}

    /// When the nick of a user changes.
    #[allow(unused_variables)]
    fn nick_change(&mut self, irc: &Irc, oldnick: &str, newnick: &str) {}

    /// When a user gets kicked from a channel.
    #[allow(unused_variables)]
    fn kick(&mut self, irc: &Irc, channel: &Channel, user: &ChannelUser) {}

    /// When the server sends a ping message.
    #[allow(unused_variables)]
    fn ping(&mut self, irc: &Irc, server: &str) {}

    /// When the server sends a pong message.
    #[allow(unused_variables)]
    fn pong(&mut self, irc: &Irc, server: &str) {}

}
