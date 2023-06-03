// SPDX-License-Identifier: Apache-2.0

use std::os::unix::io::{AsRawFd, RawFd};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use futures::Stream;
use nix::poll::{PollFd, PollFlags};

use crate::{ErrorKind, RabcClient, RabcError};

const POLL_TIMEOUT: libc::c_int = 1000; // milliseconds

#[derive(Debug)]
struct ShareState {
    waker: Option<Waker>,
}

#[derive(Debug)]
pub struct RabcClientAsync {
    client: RabcClient,
    shared_state: Arc<Mutex<ShareState>>,
}

impl RabcClientAsync {
    pub fn new() -> Result<Self, RabcError> {
        let shared_state = Arc::new(Mutex::new(ShareState { waker: None }));
        Ok(Self {
            client: RabcClient::new()?,
            shared_state,
        })
    }
}

impl Stream for RabcClientAsync {
    type Item = Result<String, RabcError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // Poll without wait
        match self.client.poll(0) {
            Ok(events) => {
                for event in events {
                    match self.client.process(&event) {
                        Ok(Some(reply)) => {
                            return Poll::Ready(Some(Ok(reply)));
                        }
                        Ok(None) => (),
                        Err(e) => {
                            return Poll::Ready(Some(Err(e)));
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("RABC client poll error: {e}");
                return Poll::Ready(Some(Err(e)));
            }
        }

        let mut shared_state = match self.shared_state.lock() {
            Ok(s) => s,
            Err(e) => {
                return Poll::Ready(Some(Err(RabcError::new(
                    ErrorKind::Bug,
                    format!(
                        "BUG: RabcClientAsync::poll_next() \
                        Failed to acquire lock on shared_state {e}",
                    ),
                ))));
            }
        };
        if shared_state.waker.is_none() {
            shared_state.waker = Some(cx.waker().clone());
            drop(shared_state);
            let fd = self.client.as_raw_fd();
            let shared_state = self.shared_state.clone();
            std::thread::spawn(move || poll_thread(fd, shared_state));
        } else {
            shared_state.waker = Some(cx.waker().clone());
            drop(shared_state);
        }
        Poll::Pending
    }
}

impl std::ops::Drop for RabcClientAsync {
    fn drop(&mut self) {
        if let Ok(mut s) = self.shared_state.lock() {
            // Signal `poll_thread()` to quit
            s.waker = None;
        }
    }
}

// This function will be invoked in a thread to notify the async executor
// via `Waker::wake()`. Will quit when `poll()` failed (except EAGAIN).
fn poll_thread(fd: RawFd, shared_state: Arc<Mutex<ShareState>>) {
    let mut poll_fds = [PollFd::new(
        fd,
        PollFlags::POLLIN
            | PollFlags::POLLOUT
            | PollFlags::POLLHUP
            | PollFlags::POLLERR,
    )];
    loop {
        if shared_state.lock().map(|s| s.waker.is_none()).ok() == Some(true) {
            std::thread::sleep(std::time::Duration::from_millis(
                POLL_TIMEOUT as u64,
            ));
        } else {
            match nix::poll::poll(&mut poll_fds, POLL_TIMEOUT) {
                // Timeout, let's check whether waker is None(client quit);
                Ok(0) => {
                    continue;
                }
                Ok(_) => match shared_state.lock() {
                    Ok(mut s) => {
                        if let Some(waker) = s.waker.take() {
                            log::debug!("poll_thread got event");
                            waker.wake();
                        } else {
                            log::debug!(
                                "poll_thread got event but Waker is None"
                            );
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "BUG: poll_thread() Failed to acquire lock: {e}"
                        );
                        return;
                    }
                },
                Err(e) => {
                    if e == nix::errno::Errno::EAGAIN {
                        continue;
                    } else {
                        log::error!(
                            "BUG: poll_thread() got error from poll(): {e}"
                        );
                        return;
                    }
                }
            }
        }
    }
}
