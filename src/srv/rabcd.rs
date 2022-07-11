// SPDX-License-Identifier: Apache-2.0

use rabc::{ErrorKind, RabcConnection, SOCKET_PATH};
use tokio::net::UnixListener;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    init_logger();

    std::fs::remove_file(SOCKET_PATH).ok();
    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind UnixListener {}: {}", SOCKET_PATH, e);
            return;
        }
    };
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    process_client(stream).await;
                });
            }
            Err(e) => {
                log::error!("Failed to accept connection {}", e);
            }
        }
    }
}

fn init_logger() {
    let mut log_builder = env_logger::Builder::new();
    // TODO: Support changing logging level
    log_builder.filter(Some("rabc"), log::LevelFilter::Debug);
    log_builder.init();
}

async fn process_client(stream: tokio::net::UnixStream) {
    log::debug!("new client connected!");
    let std_stream = match stream.into_std() {
        Ok(s) => s,
        Err(e) => {
            log::error!(
                "Failed to convert tokio::net::UnixStream to std: {}",
                e
            );
            return;
        }
    };
    let mut conn = match RabcConnection::new(std_stream) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to setup connect to client: {}", e);
            return;
        }
    };
    loop {
        match conn.ipc_recv() {
            Ok(content) => {
                log::debug!("Got content from client '{}'", content);
                if let Err(e) = conn.ipc_send("pong") {
                    log::error!("Failed to send to client: {}", e);
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::IpcConnectionError {
                    // Client disconnected
                    log::debug!("client disconnected!");
                } else {
                    log::error!("Failed to recv from client: {}", e);
                }
                break;
            }
        }
    }
}
