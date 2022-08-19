// SPDX-License-Identifier: Apache-2.0

use crate::timer::RabcTimer;
use std::os::unix::io::AsRawFd;

#[test]
fn test_timer_wait() {
    let timer = RabcTimer::new(2).unwrap();

    println!("Timer created with raw fd {}", timer.as_raw_fd());

    timer.wait().unwrap();

    println!("Timer exceeded as expected");
}
