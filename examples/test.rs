extern crate hiirc;

use hiirc::*;

struct S(Writer);

impl Listener for S {

    fn welcome(&mut self, state: &mut State) {
        self.0.join("#channel", None);
    }

    fn channel_join(&mut self, state: &mut State, channel: &str) {
        println!("{:#?}", state);
    }

}

fn main() {
    let (writer, reader) = connect("irc.freenode.net:6667", ReconnectionSettings::DoNotReconnect).unwrap();
    writer.nick("hiirc");
    writer.user("hiirc", "hiirc");
    let mut disp = Dispatcher::new(S(writer));
    for event in reader.iter() {
        println!("{:?}", event);
        disp.feed(&event);
    }
}
