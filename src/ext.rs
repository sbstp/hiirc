//! Utilities that are not part of the official IRC protocol standard.

use core::{Error, Irc, IrcWrite};

/// An extension trait to the Irc struct that adds NickServ capabilities.
///
/// Import this trait in scope and you can now use `irc.identify(password)`
pub trait NickServ {

    /// Send an identify message to the nick server.
    ///
    /// This is equivalent to /msg nickserv identify <password>.
    fn identify(&self, password: &str) -> Result<(), Error>;

}

impl NickServ for Irc {

    fn identify(&self, password: &str) -> Result<(), Error> {
        self.privmsg("nickserv", &format!("identify {}", password))
    }

}
