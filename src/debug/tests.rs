use super::{Command, Response};
use crate::{Emulator, Z80};
use serde::Deserialize;
use std::io::{ErrorKind, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

#[test]
fn server_responds() {
    env_logger::init();

    let mut emulator = Emulator::new();
    let mut cpu = Z80::new();
    let mut sock =
        TcpStream::connect(("localhost", 10000)).expect("Failed to connect to debug socket");

    assert_eq!(emulator.debug_commands_executed, 0);
    serde_json::to_writer(&sock, &Command::Version).expect("Failed to write command");
    sock.flush().expect("Failed to flush socket");
    let loop_time = Instant::now();
    loop {
        emulator.run(&mut cpu, Duration::from_millis(10));
        if emulator.debug_commands_executed >= 1 || loop_time.elapsed() > Duration::from_secs(1) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        emulator.debug_commands_executed, 1,
        "Emulator never executed version command"
    );

    let mut deserializer = serde_json::Deserializer::from_reader(&sock);
    assert_eq!(
        Response::Version(crate::built_info::PKG_VERSION.to_string()),
        Response::deserialize(&mut deserializer).expect("Failed to deserialize response")
    );
}
