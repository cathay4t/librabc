// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;

use crate::{ErrorKind, RabcError};

pub const SOCKET_PATH: &str = "/tmp/librabc";
const DEFAULT_MAX_DATA_SIZE: usize = 1024 * 1024; // 1 MiB

#[derive(Debug)]
pub struct RabcConnection {
    stream: UnixStream,
    max_size: usize,
}

impl AsRawFd for RabcConnection {
    fn as_raw_fd(&self) -> RawFd {
        self.stream.as_raw_fd()
    }
}

impl RabcConnection {
    pub fn connect() -> Result<Self, RabcError> {
        let stream = UnixStream::connect(SOCKET_PATH).map_err(|e| {
            RabcError::new(
                ErrorKind::InvalidArgument,
                format!("Failed to connect socket {}: {}", SOCKET_PATH, e),
            )
        })?;
        log::debug!("Connected to Rabc daemon {}", stream.as_raw_fd());
        Ok(Self {
            stream,
            max_size: DEFAULT_MAX_DATA_SIZE,
        })
    }

    pub fn new(stream: UnixStream) -> Result<Self, RabcError> {
        stream.set_nonblocking(false).map_err(|e| {
            RabcError::new(
                ErrorKind::Bug,
                format!("Failed to set UnixStream socket as blocking: {}", e),
            )
        })?;
        Ok(Self {
            stream,
            max_size: DEFAULT_MAX_DATA_SIZE,
        })
    }

    /// Set the max data size for IPC communication.
    pub fn set_ipc_max_size(&mut self, max_size: usize) -> &mut Self {
        self.max_size = max_size;
        self
    }

    /// Get the max data size for IPC communication.
    pub fn get_ipc_max_size(&mut self) -> usize {
        self.max_size
    }

    pub fn ipc_recv(&mut self) -> Result<String, RabcError> {
        let mut data_len_bytes = 0usize.to_ne_bytes();
        if let Err(e) = self.stream.read_exact(&mut data_len_bytes) {
            return Err(RabcError::new(
                ErrorKind::IpcConnectionError,
                format!("Failed to receive data size: {}", e),
            ));
        }
        let data_len = usize::from_ne_bytes(data_len_bytes);
        if data_len >= self.max_size {
            return Err(RabcError::new(
                ErrorKind::ExceededIpcMaxSize,
                format!(
                    "Received data exceeded the max size {} bytes, \
                     please change the limitation by set_ipc_max_size()",
                    self.max_size
                ),
            ));
        }
        let mut data = vec![0u8; data_len];
        self.stream.read_exact(data.as_mut_slice())?;
        Ok(String::from_utf8(data)?)
    }

    pub fn ipc_send(&mut self, data: &str) -> Result<(), RabcError> {
        if data.len() > self.max_size {
            return Err(RabcError::new(
                ErrorKind::ExceededIpcMaxSize,
                format!(
                    "Specified data exceeded the max size {} bytes, \
                        please change the limitation by set_ipc_max_size()",
                    self.max_size
                ),
            ));
        }
        self.stream.write_all(&data.len().to_ne_bytes())?;
        self.stream.write_all(data.as_bytes())?;
        Ok(())
    }
}
