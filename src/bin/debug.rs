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
        .subcommand(SubCommand::with_name("continue").about("Resume execution"));

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
            ("continue", _) => Command::Resume,
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
