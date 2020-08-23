use clap::{App, AppSettings, Arg, SubCommand};
use serde::Deserialize;
use std::io::{BufWriter, Write};
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
    let mut sock_responses = serde_json::Deserializer::new(serde_json::de::IoRead::new(sock));

    let mut line_editor = rustyline::Editor::<()>::with_config(
        rustyline::Config::builder().auto_add_history(true).build(),
    );
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
        .subcommand(SubCommand::with_name("version").about("Get target version information"))
        .subcommand(SubCommand::with_name("pause").about("Pause execution immediately"))
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

    loop {
        let s = line_editor.readline("tihle> ")?;
        let matches = match cli_app.get_matches_from_safe_borrow(s.split(' ')) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        let command = match matches.subcommand() {
            ("exit", _) => break,
            ("version", _) => Command::Version,
            ("pause", _) => Command::Pause,
            ("continue", _) => Command::Resume,
            ("getregs", _) => Command::GetRegisters,
            ("addbreakpoint", Some(m)) => {
                let addr = parse_address(m.value_of("address").unwrap());
                Command::AddBreakpoint(addr)
            },
            (c, _) => panic!("BUG: command {:?} is defined but not implemented", c),
        };

        serde_json::to_writer(&mut sock_commands, &command)?;
        write!(sock_commands, "\n")?;
        sock_commands.flush()?;
        let response = Response::deserialize(&mut sock_responses)?;
        println!("{:?}", response);
    }

    Ok(())
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
        Err(_) => None
    }
}

fn validate_address(s: String) -> Result<(), String> {
    match try_parse_address(&s) {
        Some(_) => Ok(()),
        None => Err("not a valid 16-bit address".into())
    }
}

fn parse_address(s: &str) -> u16 {
    try_parse_address(s).unwrap()
}