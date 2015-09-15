extern crate hiirc;

use std::borrow::ToOwned;
use std::env;

use hiirc::*;

struct Peekaboo {
    channel: String,
}

impl Peekaboo {

    pub fn new(channel: String) -> Peekaboo {
        Peekaboo {
            channel: channel,
        }
    }

}

impl Listener for Peekaboo {

    fn welcome(&mut self, irc: &mut Irc) {
        irc.join(&self.channel, None);
    }

    fn channel_join(&mut self, irc: &mut Irc, channel: &str) {
        irc.privmsg(&self.channel, "peekaboo");
        irc.quit(Some("peekaboo"));
    }

}

fn main() {
    let args: Vec<String> = env::args().collect();

    let channel = args.get(1).unwrap_or_else(|| {
        println!("Channel must be given as an argument.");
        panic!();
    });

    dispatch(Peekaboo::new(channel.to_owned()), Settings {
        addr: "irc.freenode.net:6667",
        nickname: "peekaboo",
        username: "peekaboo",
        realname: "peekaboo",
        reco_settings: None,
        mon_settings: None,
    });
}
