// SPDX-License-Identifier: Apache-2.0

use crate::{ErrorKind, RabcError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RabcEvent {
    IpcIn = 1,
    Timer,
}

impl TryFrom<u64> for RabcEvent {
    type Error = RabcError;
    fn try_from(v: u64) -> Result<Self, RabcError> {
        match v {
            x if x == Self::IpcIn as u64 => Ok(Self::IpcIn),
            x if x == Self::Timer as u64 => Ok(Self::Timer),
            _ => {
                let e = RabcError::new(
                    ErrorKind::Bug,
                    format!("Got unexpected event ID {}", v),
                );
                log::error!("{}", e);
                Err(e)
            }
        }
    }
}

impl std::fmt::Display for RabcEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IpcIn => "IpcIn",
                Self::Timer => "Timer",
            }
        )
    }
}
