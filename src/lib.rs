//! TODO
#![deny(missing_docs)]

extern crate loirc;

#[macro_use]
mod macros;
mod core;
mod listener;

pub use core::{dispatch};
pub use core::{Channel, ConnectionStatus, Error, Irc, Settings};
pub use listener::Listener;
pub use loirc::{Duration, Event, MonitorSettings, ReconnectionSettings};
