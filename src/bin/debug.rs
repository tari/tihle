use clap::{App, AppSettings, Arg, SubCommand};
use serde::Deserialize;
use std::io::{BufReader, BufWriter, Result as IoResult, Write};
use std::net::TcpStream;
use tihle::debug::{Command, Response};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("tihle debug CLI")
        .arg(
            Arg::with_name("host")
                .required(true)
                .default_value("localhost")
                .help("Remote address to connect to emulator"),
        )
        .arg(
            Arg::with_name("port")
                .required(true)
                .default_value("10000")
                .help("TCP port number to connect to emulator"),
        )
        .get_matches();

    let hostname = matches.value_of("host").unwrap();
    let port = matches.value_of("port").unwrap().parse::<u16>()?;
    let sock = TcpStream::connect((hostname, port))?;
    let mut sock_commands = BufWriter::new(sock.try_clone()?);

    // Handle IO in a separate thread so we can wait for either input or an interrupt.
    // This works because interrupts don't send responses, so the IO thread is always in a strict
    // command-response loop.
    let (input_tx, input_rx) = crossbeam_channel::bounded::<Command>(0);
    std::thread::spawn(move || handle_io(input_tx, BufReader::new(sock)));

    // On ^C send an interrupt command
    let (interrupt_tx, interrupt_rx) = crossbeam_channel::bounded::<()>(0);
    ctrlc::set_handler(move || {
        interrupt_tx.send(()).unwrap();
    })
    .expect("Unable to register interrupt handler");

    loop {
        let command = crossbeam_channel::select! {
            recv(input_rx) -> msg => match msg {
                // Input thread closed the channel, meaning we should exit
                Err(_) => return Ok(()),
                Ok(msg) => {
                    assert_ne!(msg, Command::Interrupt, "Interrupts don't send responses");
                    msg
                }
            },
            recv(interrupt_rx) -> _ => Command::Interrupt,
        };

        send_command(&mut sock_commands, &command)?;
        if command == Command::Interrupt {
            // Interrupt commands don't get any response.
            continue;
        }
    }
}

fn send_command<W: Write>(mut w: W, command: &Command) -> IoResult<()> {
    serde_json::to_writer(&mut w, command)?;
    write!(&mut w, "\n")?;
    w.flush()
}

fn try_parse_address(s: &str) -> Option<u16> {
    let (s, base) = if s.starts_with("$") {
        (&s[1..], 16)
    } else if s.starts_with("0x") || s.starts_with("0X") {
        (&s[2..], 16)
    } else if s.starts_with("0b") || s.starts_with("0B") {
        (&s[2..], 2)
    } else {
        (&s[..], 10)
    };

    match u16::from_str_radix(s, base) {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

fn validate_address(s: String) -> Result<(), String> {
    match try_parse_address(&s) {
        Some(_) => Ok(()),
        None => Err("not a valid 16-bit address".into()),
    }
}

fn parse_address(s: &str) -> u16 {
    try_parse_address(s).unwrap()
}

fn handle_io<R: std::io::Read>(commands_tx: crossbeam_channel::Sender<Command>, input_stream: R) {
    let mut cli_app = App::new("tihle debugger CLI")
        .usage("Type commands, send them to the emulator")
        .settings(&[
            AppSettings::DisableVersion,
            AppSettings::SubcommandRequired,
            AppSettings::VersionlessSubcommands,
        ])
        .global_settings(&[
            AppSettings::NoBinaryName,
            AppSettings::DisableHelpFlags,
            AppSettings::InferSubcommands,
        ])
        .subcommand(
            SubCommand::with_name("exit")
                .about("Exit the debugger")
                .alias("quit"),
        )
        .subcommand(SubCommand::with_name("continue").about("Resume execution"))
        .subcommand(SubCommand::with_name("getregs").about("Get the values of all registers"))
        .subcommand(
            SubCommand::with_name("addbreakpoint")
                .alias("mkbp")
                .about("Add a breakpoint")
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .validator(validate_address)
                        .help("Address to break at"),
                ),
        );

    let mut deserializer = serde_json::Deserializer::new(serde_json::de::IoRead::new(input_stream));

    // Synchronize with the emulator by sending a Sync command with pseudo-random token and pulling
    // responses until we get the same token back.
    eprintln!("Synchronizing with emulator; press Ctrl+C to break, or\n\
               press Pause/Break in emulator or exit the current program");
    let token = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs_f64();
    commands_tx.send(Command::Sync(token)).unwrap();
    loop {
        let response = Response::deserialize(&mut deserializer).expect("Failed to deserialize sync response");
        match response {
            Response::SyncResponse(token_rx, ref version) if token_rx == token => {
                eprintln!("Emulator reports version {}", version);
                break;
            }
            r => eprintln!("{:?}", r),
        }
    }

    let mut line_editor = rustyline::Editor::<()>::with_config(
        rustyline::Config::builder().auto_add_history(true).build(),
    );
    loop {
        let s = line_editor
            .readline("tihle> ")
            .expect("Failed to read input");
        let matches = match cli_app.get_matches_from_safe_borrow(s.split(' ')) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        let command = match matches.subcommand() {
            ("exit", _) => break,
            ("continue", _) => Command::Run,
            ("getregs", _) => Command::GetRegisters,
            ("addbreakpoint", Some(m)) => {
                let addr = parse_address(m.value_of("address").unwrap());
                Command::AddBreakpoint(addr)
            }
            (c, _) => panic!("BUG: command {:?} is defined but not implemented", c),
        };

        commands_tx.send(command).unwrap();
        let response = Response::deserialize(&mut deserializer)
            .expect("Failed to deserialize emulator response");
        println!("{:?}", response);
    }
}
