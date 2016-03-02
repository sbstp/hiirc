#![allow(unused_must_use)]

extern crate hiirc;

use std::env;
use std::sync::Arc;

use hiirc::*;

/// This is the struct on which the listener is implemented.
struct Peekaboo<'a> {
    channel: &'a str,
}

impl<'a> Peekaboo<'a> {

    pub fn new(channel: &str) -> Peekaboo {
        Peekaboo {
            channel: channel,
        }
    }

}

impl<'a> Listener for Peekaboo<'a> {

    /// On any event we receive, print the Debug of it.
    fn any(&mut self, _: Arc<Irc>, event: &Event) {
        println!("{:?}", &event);
    }

    /// When the welcome message is received, join the channel.
    fn welcome(&mut self, irc: Arc<Irc>) {
        irc.join(self.channel, None);
    }

    /// When the channel is joined, say "peekaboo" and quit.
    fn channel_join(&mut self, irc: Arc<Irc>, channel: Arc<Channel>) {
        irc.privmsg(channel.name(), "peekaboo");
        irc.quit(Some("peekaboo"));
    }

}

fn main() {
    let args: Vec<String> = env::args().collect();
    let channel = args.get(1).expect("Channel must be given as an argument.");

    Settings::new("irc.freenode.net:6667", "peekaboo")
        .username("peekaboo")
        .realname("peekaboo")
        .dispatch(Peekaboo::new(channel)).unwrap();

    /* This code is equivalent to the builder API shown above.
    let mut settings = Settings::new("irc.freenode.net:6667", "peekaboo");
    settings.username = "peekaboo";
    settings.realname = "peekaboo";

    dispatch(Peekaboo::new(channel), settings).unwrap();
    */
}
