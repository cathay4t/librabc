// SPDX-License-Identifier: Apache-2.0

use std::os::unix::io::{AsRawFd, RawFd};

use nix::sys::time::{TimeSpec, TimeValLike};
use nix::sys::timerfd::{
    ClockId::CLOCK_BOOTTIME, Expiration, TimerFd, TimerFlags, TimerSetTimeFlags,
};

use crate::{ErrorKind, RabcError};

#[derive(Debug)]
pub(crate) struct RabcTimer {
    pub(crate) fd: TimerFd,
}

impl AsRawFd for RabcTimer {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl RabcTimer {
    pub(crate) fn new(time: u32) -> Result<Self, RabcError> {
        let fd =
            TimerFd::new(CLOCK_BOOTTIME, TimerFlags::empty()).map_err(|e| {
                let e = RabcError::new(
                    ErrorKind::Bug,
                    format!("Failed to create timerfd {}", e),
                );
                log::error!("{}", e);
                e
            })?;
        fd.set(
            Expiration::Interval(TimeSpec::seconds(time.into())),
            TimerSetTimeFlags::empty(),
        )
        .map_err(|e| {
            let e = RabcError::new(
                ErrorKind::Bug,
                format!("Failed to set timerfd {}", e),
            );
            log::error!("{}", e);
            e
        })?;
        log::debug!("TimerFd created {:?} with {} seconds", fd, time);
        Ok(Self { fd })
    }

    pub(crate) fn wait(&self) -> Result<(), RabcError> {
        if let Err(e) = self.fd.wait() {
            let e = RabcError::new(
                ErrorKind::Bug,
                format!("Failed to wait timerfd {}", e),
            );
            log::error!("{}", e);
            Err(e)
        } else {
            Ok(())
        }
    }
}
