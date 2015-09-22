#![deny(missing_docs)]
//! `hiirc` is a library built on top of [loirc](https://github.com/SBSTP/loirc). The goal
//! is to offer a friendly, event-based API.
//!
//! To use the library, implement the listener trait and give an instance of your implementation
//! to the dispatch function, accompanied with an instance of the Settings struct configured to
//! your needs.

extern crate loirc;

#[macro_use]
mod macros;
mod core;
pub mod ext;
mod listener;

pub use core::{dispatch};
pub use core::{Channel, ConnectionStatus, Error, Irc, Settings, ChannelUser, ChannelUserStatus};
pub use listener::Listener;
pub use loirc::Error as LoircError;
pub use loirc::{Code, Duration, Event, Message, MonitorSettings, ParseError, Prefix, PrefixUser,
                ReconnectionSettings};
