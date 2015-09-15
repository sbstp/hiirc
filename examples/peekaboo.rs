#![allow(unused_must_use)]
extern crate hiirc;
use std::env;
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
    fn any(&mut self, _: &mut Irc, event: &Event) {
        println!("{:?}", &event);
    }

    /// When the welcome message is received, join the channel.
    fn welcome(&mut self, irc: &mut Irc) {
        irc.join(self.channel, None);
    }

    /// When the channel is joined, say "peekaboo" and quit.
    fn channel_join(&mut self, irc: &mut Irc, channel: &str) {
        irc.privmsg(channel, "peekaboo");
        irc.quit(Some("peekaboo"));
    }

}

fn main() {
    let args: Vec<String> = env::args().collect();
    let channel = args.get(1).expect("Channel must be given as an argument.");

    dispatch(Peekaboo::new(channel), Settings {
        addr: "irc.freenode.net:6667",
        nickname: "peekaboo",
        username: "peekaboo",
        realname: "peekaboo",
        reco_settings: None,
        mon_settings: None,
    }).expect("Failed to connect to server.");
}
