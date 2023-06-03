// SPDX-License-Identifier: Apache-2.0

use std::os::unix::io::{AsRawFd, RawFd};

use crate::{
    epoll::RabcEpoll, timer::RabcTimer, RabcConnection, RabcError, RabcEvent,
};

const DEFAULT_TIMER_INTERVAL: u32 = 2; // send out ping every 2 seconds

#[derive(Debug)]
pub struct RabcClient {
    timer: RabcTimer,
    conn: RabcConnection,
    epoll: RabcEpoll,
}

impl AsRawFd for RabcClient {
    fn as_raw_fd(&self) -> RawFd {
        self.epoll.fd
    }
}

impl RabcClient {
    pub fn new() -> Result<Self, RabcError> {
        let epoll = RabcEpoll::new()?;
        let timer = RabcTimer::new(DEFAULT_TIMER_INTERVAL)?;
        epoll.add_fd(timer.as_raw_fd(), RabcEvent::Timer)?;
        let conn = RabcConnection::connect()?;
        epoll.add_fd(conn.as_raw_fd(), RabcEvent::IpcIn)?;

        Ok(Self { timer, conn, epoll })
    }

    pub fn poll(
        &mut self,
        wait_time: u32,
    ) -> Result<Vec<RabcEvent>, RabcError> {
        self.epoll.poll(wait_time)
    }

    pub fn process(
        &mut self,
        event: &RabcEvent,
    ) -> Result<Option<String>, RabcError> {
        log::debug!("Processing event {:?}", event);
        match event {
            RabcEvent::Timer => {
                self.timer.wait()?;
                self.conn.ipc_send("ping")?;
                Ok(None)
            }
            RabcEvent::IpcIn => {
                let reply = self.conn.ipc_recv()?;
                Ok(Some(reply))
            }
        }
    }
}
