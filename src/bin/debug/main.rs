#[macro_use]
extern crate pest_derive;

mod expr;

use crate::expr::CpuContext;
use clap::{App, AppSettings, Arg, SubCommand};
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, BufWriter, Result as IoResult, Write};
use std::net::TcpStream;
use tihle::debug::{Command, PauseReason, Response};

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
                Ok(msg) => msg
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

/// Container for the connection to the emulator.
///
/// Provides some convenience methods for sending and receiving commands/responses.
struct Remote<R>
where
    R: std::io::Read,
{
    tx: crossbeam_channel::Sender<Command>,
    rx: serde_json::Deserializer<serde_json::de::IoRead<R>>,
}

impl<R: std::io::Read> Remote<R> {
    fn send(&mut self, command: Command) {
        self.tx.send(command).expect("Command channel disconnected")
    }

    fn recv(&mut self) -> Response {
        Response::deserialize(&mut self.rx).expect("Failed to deserialize response")
    }

    fn exec(&mut self, command: Command) -> Response {
        self.send(command);
        self.recv()
    }

    /// Synchronize with the emulator by sending a Sync command with pseudo-random token and pulling
    /// responses until we get the same token back.
    ///
    /// This ensures that when we connect the emulator isn't in an unexpected state (like sending
    /// a response to a command that it received before we connected).
    fn sync(&mut self) {
        // Interrupt to ensure our commands will actually be processed, and because pausing on
        // connect is a sensible behavior.
        self.send(Command::Interrupt);

        let token = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        self.send(Command::Sync(token));
        loop {
            match self.recv() {
                Response::SyncResponse(token_rx, ref version) if token_rx == token => {
                    eprintln!("Synchronized. Emulator reports version {}", version);
                    break;
                }
                r => eprintln!("{:?}", r),
            }
        }
    }

    /// Send a wait command and block until emulation pauses.
    fn wait(&mut self) {
        let message = match self.exec(Command::Wait) {
            Response::WaitResult(reason) => match reason {
                PauseReason::Interrupted => Some("break requested"),
                PauseReason::Breakpoint => Some("hit breakpoint"),
                PauseReason::Terminated => Some("program terminated"),
                PauseReason::AlreadyPaused => None,
            },
            r => panic!("Unexpected wait response: {:?}", r),
        };

        let regs = match self.exec(Command::GetRegisters) {
            Response::RegisterValues(regs) => regs,
            r => panic!("Unexpected GetRegisters response: {:?}", r),
        };
        eprint!("Emulation paused at {:#06X}", regs.pc);
        if let Some(m) = message {
            eprintln!("{}", m);
        } else {
            eprintln!();
        }
    }
}

fn handle_io<R: std::io::Read>(commands_tx: crossbeam_channel::Sender<Command>, input_stream: R) {
    let mut cli_parser = make_cli_parser();
    let mut remote = Remote {
        tx: commands_tx,
        rx: serde_json::Deserializer::new(serde_json::de::IoRead::new(input_stream)),
    };
    remote.sync();

    let mut line_editor = rustyline::Editor::<()>::with_config(
        rustyline::Config::builder().auto_add_history(true).build(),
    );
    let expr_parser = expr::ExpressionParser::new();
    loop {
        // For predictable behavior, only accept commands while the CPU is paused.
        remote.wait();

        let s = line_editor
            .readline("tihle> ")
            .expect("Failed to read input");
        let matches = match cli_parser.get_matches_from_safe_borrow(s.split(' ')) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        let command = match matches.subcommand() {
            ("exit", _) => break,
            ("continue", _) => Command::Run,
            ("addbreakpoint", Some(m)) => {
                let mut ctx = ExpressionContext::new(&mut remote);
                match expr_parser.evaluate(&mut ctx, m.value_of("address").unwrap()) {
                    Ok(addr) => Command::AddBreakpoint(addr),
                    Err(e) => {
                        println!("Malformed address expression: {}", e);
                        continue;
                    }
                }
            }
            ("removebreakpoint", Some(m)) => {
                let mut ctx = ExpressionContext::new(&mut remote);
                match expr_parser.evaluate(&mut ctx, m.value_of("address").unwrap()) {
                    Ok(addr) => Command::RemoveBreakpoint(addr),
                    Err(e) => {
                        println!("Malformed address expression: {}", e);
                        continue;
                    }
                }
            }
            ("print", Some(m)) => {
                let mut ctx = ExpressionContext::new(&mut remote);
                let exprs: String = m.values_of("expression").unwrap().collect();
                for expr in exprs.split(',') {
                    match expr_parser.evaluate(&mut ctx, expr) {
                        Ok(value) => {
                            print!("{} => ", expr);
                            if m.is_present("d") {
                                println!("{}", value)
                            } else if m.is_present("b") {
                                println!("{:#b}", value);
                            } else {
                                println!("{:#x}", value);
                            }
                        }
                        Err(e) => println!("Unable to evaluate expression: {}", e),
                    }
                }
                continue;
            }
            ("hexdump", Some(m)) => {
                let expr: String = m.values_of("address").unwrap().collect();
                let mut ctx = ExpressionContext::new(&mut remote);
                let addr = match expr_parser.evaluate(&mut ctx, &expr) {
                    Ok(a) => a,
                    Err(e) => {
                        println!("Unable to evaluate address expression: {}", e);
                        continue;
                    }
                };
                let count = u16::from_str_radix(m.value_of("count").unwrap(), 10).unwrap();

                let data = ctx.read_memory(addr, count);
                for (chunk_no, row) in data.chunks(16).enumerate() {
                    print!("{:04X} ", addr.wrapping_add((row.len() * chunk_no) as u16));
                    for byte in row {
                        print!("{:02X} ", byte);
                    }
                    println!();
                }
                continue;
            }
            ("dump", Some(m)) => {
                let mut ctx = ExpressionContext::new(&mut remote);
                let addr = match expr_parser.evaluate(&mut ctx, m.value_of("address").unwrap()) {
                    Ok(a) => a,
                    Err(e) => {
                        println!("Unable to evaluate address expression: {}", e);
                        continue;
                    }
                };
                let size = match expr_parser.evaluate(&mut ctx, m.value_of("size").unwrap()) {
                    Ok(a) => a,
                    Err(e) => {
                        println!("Unable to evaluate size expression: {}", e);
                        continue;
                    }
                };
                let data = ctx.read_memory(addr, size);

                let filename = m.value_of("filename").unwrap();
                let mut f = match File::create(filename) {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Unable to open {:?} for writing: {}", filename, e);
                        continue;
                    }
                };
                if let Err(e) = f.write_all(&data) {
                    println!("Failed to write to file: {}", e);
                } else {
                    println!("Wrote {} bytes to {:?}", size, filename);
                }
                continue;
            }
            (c, _) => panic!("BUG: command {:?} is defined but not implemented", c),
        };

        println!("{:?}", remote.exec(command));
    }
}

fn make_cli_parser() -> App<'static, 'static> {
    fn validate_u16(s: String) -> Result<(), String> {
        match u16::from_str_radix(&s, 10) {
            Ok(_) => Ok(()),
            Err(_) => Err("not a valid 16-bit integer".into()),
        }
    }

    App::new("tihle debugger CLI")
        .usage(
            "Type commands, send them to the emulator

Where expressions are accepted, use syntax similar to typical Z80 assembly:
 * Register names verbatim (a, hl, iy, bc'...).
 * Literal numbers as decimal (255), hex (0xFF, $FF) or binary (%11111111,
   0b11111111).
 * Arithmetic: add (+), subtract (-), multiply (*), divide (/), remainder (%).
   Operators are parsed strictly left to right.
 * Memory contents in parentheses: (hl + 2). Size of memory access is 8 bits
   by default; prefix with 'w' to do word access: w(hl + 2).",
        )
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
        .subcommand(
            SubCommand::with_name("print")
                .about("Display the value of an expression")
                .alias("p")
                .arg(
                    Arg::with_name("d")
                        .short("d")
                        .conflicts_with("d")
                        .help("Print value as decimal"),
                )
                .arg(
                    Arg::with_name("b")
                        .short("b")
                        .conflicts_with("d")
                        .help("Print value as binary"),
                )
                .arg(
                    Arg::with_name("expression")
                        .required(true)
                        .multiple(true)
                        .help("Expressions to evaluate, comma-separated"),
                ),
        )
        .subcommand(
            SubCommand::with_name("addbreakpoint")
                .alias("mkbp")
                .about("Add a breakpoint at the given address")
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .help("Expression evaluating to the target address"),
                ),
        )
        .subcommand(
            SubCommand::with_name("removebreakpoint")
                .alias("rmbp")
                .about("Remove a breakpoint at the given address")
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .help("Expression evaluating to the target address"),
                ),
        )
        .subcommand(
            SubCommand::with_name("hexdump")
                .alias("x")
                .about("Display the contents of memory at a given address")
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .multiple(true)
                        .help("Expression evaluating to the target address"),
                )
                .arg(
                    Arg::with_name("count")
                        .short("n")
                        .takes_value(true)
                        .default_value("1")
                        .validator(validate_u16)
                        .help("Number of bytes to display after address"),
                ),
        )
        .subcommand(
            SubCommand::with_name("dump")
                .about("Save memory contents to a file")
                .arg(Arg::with_name("filename").required(true))
                .arg(Arg::with_name("address").required(true))
                .arg(Arg::with_name("size").required(true)),
        )
}

struct ExpressionContext<'a, RIO>
where
    RIO: std::io::Read,
{
    remote: &'a mut Remote<RIO>,
    regs: Option<tihle::debug::Registers>,
}

impl<'a, RIO: std::io::Read> ExpressionContext<'a, RIO> {
    pub fn new(remote: &'a mut Remote<RIO>) -> Self {
        ExpressionContext { remote, regs: None }
    }

    fn get_regs(&mut self) -> &tihle::debug::Registers {
        if self.regs.is_none() {
            match self.remote.exec(Command::GetRegisters) {
                Response::RegisterValues(regs) => self.regs = Some(regs),
                r => panic!("Unexpected response to GetRegisters: {:?}", r),
            }
        }
        self.regs.as_ref().unwrap()
    }
}

impl<'a, RIO: std::io::Read> expr::CpuContext for ExpressionContext<'a, RIO> {
    fn get_register(&mut self, r: expr::RegisterName) -> u16 {
        use expr::RegisterName::*;

        let regs = self.get_regs();
        match r {
            AF => regs.af,
            BC => regs.bc,
            DE => regs.de,
            HL => regs.hl,
            IX => regs.ix,
            IY => regs.iy,
            AF_ => regs.af_,
            BC_ => regs.bc_,
            DE_ => regs.de_,
            HL_ => regs.hl_,
            A => regs.af >> 8,
            F => regs.af & 0xFF,
            B => regs.bc >> 8,
            C => regs.bc & 0xFF,
            D => regs.de >> 8,
            E => regs.de & 0xFF,
            H => regs.hl >> 8,
            L => regs.hl & 0xFF,
            PC => regs.pc,
            SP => regs.sp,
            I => regs.i as u16,
            R => regs.r as u16,
        }
    }

    fn read_memory(&mut self, addr: u16, size: u16) -> Vec<u8> {
        let data = match self.remote.exec(Command::ReadMem { addr, size }) {
            Response::Memory(data) => data,
            r => panic!("Unexpected response to ReadMem: {:?}", r),
        };
        assert_eq!(
            data.len(),
            size as usize,
            "Received data was different length than requested"
        );
        data
    }
}
