use super::{Command, Debugger, Response};
use crate::{Emulator, Z80};
use std::time::{Duration, Instant};

#[test]
fn server_responds() {
    env_logger::init();

    let mut emulator = Emulator::new();
    let mut cpu = Z80::new();
    let (mut debugger, commands, responses) = Debugger::create_for_test();

    assert_eq!(debugger.commands_executed, 0);
    commands.send(Command::Version).unwrap();
    let loop_time = Instant::now();
    loop {
        emulator.run(&mut cpu, Some(&mut debugger), Duration::from_millis(10));
        if debugger.commands_executed >= 1 || loop_time.elapsed() > Duration::from_secs(1) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        debugger.commands_executed, 1,
        "Emulator never executed version command"
    );

    let response = responses.recv().expect("Unable to receive response");
    assert_eq!(
        Response::Version(crate::built_info::PKG_VERSION.to_string()),
        response
    );
}
