use loirc::Event;

use {Channel, Irc};

/// Implement this trait to handle events.
pub trait Listener {

    /// Any event.
    #[allow(unused_variables)]
    fn any(&mut self, irc: &Irc, event: &Event) {}

    /// When the server sends the welcome packet.
    #[allow(unused_variables)]
    fn welcome(&mut self, irc: &Irc) {}

    /// When the client sucessfully joins a channel.
    #[allow(unused_variables)]
    fn channel_join(&mut self, irc: &Irc, channel: &str) {}

    /// When a user joins a channel we are listening on.
    #[allow(unused_variables)]
    fn user_join(&mut self, irc: &Irc, channel: &str, nickname :&str) {}

    /// When a user parts a channel we are listening on.
    #[allow(unused_variables)]
    fn user_part(&mut self, irc: &Irc, channel: &str, nickname :&str) {}

    /// When a user quits.
    #[allow(unused_variables)]
    fn user_quit(&mut self, irc: &Irc, nickname: &str) {}

    /// When a channel message is received.
    #[allow(unused_variables)]
    fn channel_msg(&mut self, irc: &Irc, sender: &str, channel: &str, message: &str) {}

    /// When a private message is received.
    #[allow(unused_variables)]
    fn private_msg(&mut self, irc: &Irc, sender: &str, message: &str) {}

    /// Reply to a topic command or when joining a channel.
    ///
    /// Note that this event might be called before the channel's username list is populated.
    /// If a channel has no topic, this event will not be fired when you join a channel.
    /// It's safe to assume that a channel has no topic if this event is not fired when joining.
    /// If you use the `topic` method however, you will always get a result.
    #[allow(unused_variables)]
    fn topic(&mut self, irc: &Irc, channel: &Channel, topic: Option<&str>) {}

    /// When the topic of is changed by someone.
    #[allow(unused_variables)]
    fn topic_change(&mut self, irc: &Irc, channel: &Channel, topic: Option<&str>) {}

}
