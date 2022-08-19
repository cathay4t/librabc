// SPDX-License-Identifier: Apache-2.0

mod client;
mod epoll;
mod error;
mod event;
mod ipc;
mod timer;
mod unit_tests;

pub use crate::client::RabcClient;
pub use crate::error::{ErrorKind, RabcError};
pub use crate::event::RabcEvent;
pub use crate::ipc::{RabcConnection, SOCKET_PATH};
