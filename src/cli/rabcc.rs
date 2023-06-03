// SPDX-License-Identifier: Apache-2.0

use futures::stream::StreamExt;
use rabc::RabcClientAsync;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    let mut client = RabcClientAsync::new()?;
    for i in 0..10 {
        if let Some(reply) = client.next().await {
            log::info!("{i}: Got reply from daemon: {}", reply?);
        }
    }
    Ok(())
}

fn init_logger() {
    let mut log_builder = env_logger::Builder::new();
    // TODO: Support changing logging level
    log_builder.filter(Some("rabc"), log::LevelFilter::Debug);
    log_builder.init();
}
