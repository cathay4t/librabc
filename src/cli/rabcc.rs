// SPDX-License-Identifier: Apache-2.0

use rabc::RabcClient;

const WAIT_TIME: u32 = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    let mut client = RabcClient::new()?;
    for _ in 0..10 {
        let events = client.poll(WAIT_TIME)?;
        println!("Got events {:?}", events);
        for event in events {
            log::debug!("Got event {}", event);
            if let Some(reply) = client.process(&event)? {
                println!("Got reply from daemon {:?}", reply);
            }
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
