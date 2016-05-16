/*
 * This example imitates the behavior of a friend of mine that often times out on irc.
 * The server often lets you know that he hasn't received any message from the past 5
 * minutes (time out). This program simulates the behavior by not replying to ping
 * messages sent by the server. The server ends up closing the connection since it
 * believes it's dead. The program will reconnect to the server right after disconnecting.
 * The behavior is triggered when you speak to the bot using its nickname.
 */
#![allow(unused_variables)]
#![allow(unused_must_use)]

extern crate hiirc;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use hiirc::*;

struct Fifteen {
    reply_to_ping: bool,
}

static NICKNAME: &'static str = "FifteenIsTimeout";
static USERNAME: &'static str = "hiirc";
static REALNAME: &'static str = "Fifteen should stop timing out";

impl Listener for Fifteen {

    fn any(&mut self, irc: Arc<Irc>, event: &Event) {
        println!("{:?}", event);
    }

    fn channel_msg(&mut self, irc: Arc<Irc>, channel: Arc<Channel>, user: Arc<ChannelUser>, msg: &str) {
        if msg.starts_with(NICKNAME) {
            self.reply_to_ping = false;
            thread::spawn(move || {

                thread::sleep(std::time::Duration::from_secs(2));
                irc.close();
                println!("closezo");
            });
            println!("donezo");
        }
    }

    fn ping(&mut self, irc: Arc<Irc>, server: &str) {
        if self.reply_to_ping {
            irc.pong(server);
        }
    }

    fn reconnect(&mut self, irc: Arc<Irc>) {
        self.reply_to_ping = true;
    }

    fn welcome(&mut self, irc: Arc<Irc>) {
        irc.join("#SexManiac", None);
    }

}

fn main() {
    Settings::new("irc.freenode.net:6667", NICKNAME)
        .username(USERNAME)
        .realname(REALNAME)
        .reconnection(ReconnectionSettings::Reconnect {
            max_attempts: 0,
            delay_between_attempts: Duration::from_secs(5),
            delay_after_disconnect: Duration::from_secs(15),
        })
        .auto_ping(false)
        .dispatch(Fifteen{ reply_to_ping: true }).unwrap();

    /* This code is equivalent to the builder API shown above.
    let mut settings = Settings::new("irc.freenode.net:6667", NICKNAME);
    settings.username = USERNAME;
    settings.realname = REALNAME;
    settings.reconnection = ReconnectionSettings::Reconnect {
        max_attempts: 0,
        delay_between_attempts: Duration::seconds(5),
        delay_after_disconnect: Duration::seconds(15),
    };
    settings.auto_ping = false;

    dispatch(Fifteen{ reply_to_ping: true }, settings).unwrap();
    */
}
