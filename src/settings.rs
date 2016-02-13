use encoding::EncodingRef;
use encoding::all::UTF_8;
use loirc::{MonitorSettings, ReconnectionSettings};
use ::core::{dispatch, Error};
use ::listener::Listener;

/// Settings for the dispatcher.
pub struct Settings<'a> {
    /// Address of the irc server.
    pub addr: &'a str,
    /// Preferred nickname.
    pub nickname: &'a str,
    /// Username.
    pub username: &'a str,
    /// Real name.
    pub realname: &'a str,
    /// Reconnection settings. If None, reconnection is disabled.
    pub reconnection: ReconnectionSettings,
    /// Monitor settings. If None, monitoring is disabled.
    pub monitor: Option<MonitorSettings>,
    /// Automatically identify after reconnection.
    pub auto_ident: bool,
    /// Automatically reply to ping requests.
    pub auto_ping: bool,
    /// Encoding used for the connection.
    pub encoding: EncodingRef,
    /// Server password
    pub password: &'a str,
}

impl<'a> Settings<'a> {

    /// Create new settings with sensible default values.
    ///
    /// The default values are:
    ///
    /// ```ignore
    /// username: "hiirc",
    /// realname: "hiirc",
    /// reconnection: ReonnectionSettings::DoNotReconnect,
    /// monitor: None,
    /// auto_ident: true,
    /// auto_ping: true,
    /// encoding: UTF_8,
    /// ```
    pub fn new<'b>(addr: &'b str, nickname: &'b str) -> Settings<'b> {
        Settings {
            addr: addr,
            nickname: nickname,
            username: "hiirc",
            realname: "hiirc",
            reconnection: ReconnectionSettings::DoNotReconnect,
            monitor: None,
            auto_ident: true,
            auto_ping: true,
            encoding: UTF_8,
            password: "",
        }
    }

    /// Modify the username.
    pub fn username(mut self, username: &'a str) -> Settings<'a> {
        self.username = username;
        self
    }

    /// Modify the realname.
    pub fn realname(mut self, realname: &'a str) -> Settings<'a> {
        self.realname = realname;
        self
    }

    /// Modify the reconnection settings.
    pub fn reconnection(mut self, reconnection: ReconnectionSettings) -> Settings<'a> {
        self.reconnection = reconnection;
        self
    }

    /// Modify the monitor settings.
    pub fn monitor(mut self, monitor: Option<MonitorSettings>) -> Settings<'a> {
        self.monitor = monitor;
        self
    }

    /// Enable/disable automatic indentification.
    pub fn auto_ident(mut self, auto_ident: bool) -> Settings<'a> {
        self.auto_ident = auto_ident;
        self
    }

    /// Enable/disable automatic ping replies.
    pub fn auto_ping(mut self, auto_ping: bool) -> Settings<'a> {
        self.auto_ping = auto_ping;
        self
    }

    /// Modify the encoding used for this connection.
    pub fn encoding(mut self, encoding: EncodingRef) -> Settings<'a> {
        self.encoding = encoding;
        self
    }

    /// Modify the server password.
    pub fn password(mut self, password: &'a str) -> Settings<'a> {
        self.password = password;
        self
    }

    /// Connect to the server and begin dispatching events using the given `Listener`.
    pub fn dispatch<L>(self, listener: L) -> Result<(), Error>
        where L: Listener
    {
        dispatch(listener, self)
    }

}
