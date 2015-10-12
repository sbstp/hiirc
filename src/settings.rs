use encoding::EncodingRef;
use encoding::all::UTF_8;
use loirc::{MonitorSettings, ReconnectionSettings};

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
}

impl<'a> Settings<'a> {

    /// Create new settings with sensible defaults.
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
        }
    }

}
