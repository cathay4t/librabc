// SPDX-License-Identifier: Apache-2.0

use std::os::unix::io::RawFd;

use nix::sys::epoll::{
    epoll_create, epoll_ctl, epoll_wait, EpollEvent, EpollFlags, EpollOp,
};

use crate::{event::RabcEvent, ErrorKind, RabcError};

const EVENT_BUFFER_COUNT: usize = 16;

#[derive(Clone, Debug)]
pub(crate) struct RabcEpoll {
    pub(crate) fd: RawFd,
}

impl RabcEpoll {
    pub(crate) fn new() -> Result<Self, RabcError> {
        Ok(Self {
            fd: epoll_create().map_err(|e| {
                let e = RabcError::new(
                    ErrorKind::Bug,
                    format!("Failed to epoll_create(): {}", e),
                );
                log::error!("{}", e);
                e
            })?,
        })
    }

    pub(crate) fn add_fd(
        &self,
        fd: RawFd,
        event: RabcEvent,
    ) -> Result<(), RabcError> {
        log::debug!("Adding fd {} to Epoll {}, event {}", fd, self.fd, event);
        let event = EpollEvent::new(EpollFlags::EPOLLIN, event as u64);
        epoll_ctl(self.fd, EpollOp::EpollCtlAdd, fd, &mut Some(event)).map_err(
            |e| {
                let e = RabcError::new(
                    ErrorKind::Bug,
                    format!(
                        "Failed to epoll_ctl({}, {:?}, {}, {:?}): {}",
                        self.fd,
                        EpollOp::EpollCtlAdd,
                        fd,
                        event,
                        e
                    ),
                );
                log::error!("{}", e);
                e
            },
        )
    }

    pub(crate) fn poll(
        &self,
        wait_time: u32,
    ) -> Result<Vec<RabcEvent>, RabcError> {
        let mut events: [EpollEvent; EVENT_BUFFER_COUNT] =
            [EpollEvent::empty(); EVENT_BUFFER_COUNT];

        let wait_time: isize = (1000 * wait_time).try_into().map_err(|e| {
            RabcError::new(
                ErrorKind::InvalidArgument,
                format!("wait time too big {}: {}", wait_time, e),
            )
        })?;

        let changed_count = epoll_wait(self.fd, &mut events, wait_time)
            .map_err(|e| {
                let e = RabcError::new(
                    ErrorKind::Bug,
                    format!("Failed on epoll_wait(): {}", e),
                );
                log::error!("{}", e);
                e
            })?;
        let mut ret = Vec::new();
        for i in &events[..changed_count] {
            ret.push(RabcEvent::try_from(i.data())?);
        }
        Ok(ret)
    }
}
