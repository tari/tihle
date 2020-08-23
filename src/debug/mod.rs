use crate::{Emulator, Z80};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufWriter, Result as IoResult};
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Command {
    /// Pause execution until resumed.
    Pause,
    /// Resume execution, such as after a breakpoint or pause command
    Resume,
    /// Return the emulator version, [Response::Version]
    Version,
    /// Return the value of all registers, [Response::RegisterValues]
    GetRegisters,
    /// Set the value of all registers
    SetRegisters(Registers),
    /// Add a breakpoint
    AddBreakpoint(u16),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Response {
    /// The command completed with no interesting output
    Ok,
    /// The command was invalid and has been ignored.
    Invalid(&'static str),
    /// The command is not implemented.
    NotImplemented,
    Version(String),
    RegisterValues(Registers),
}

#[derive(Debug)]
pub struct Debugger {
    commands_in: Receiver<Command>,
    responses_out: Sender<Response>,
    commands_executed: u32,
    paused: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Registers {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub ix: u16,
    pub iy: u16,
    pub pc: u16,
    pub sp: u16,
    pub i: u8,
    pub r: u8,
    pub af_: u16,
    pub bc_: u16,
    pub de_: u16,
    pub hl_: u16,
}

impl Debugger {
    pub fn create<A: ToSocketAddrs>(listen_addr: A) -> Self {
        let (thread_out, emu_in) = channel();
        let (emu_out, thread_in) = channel();

        let listener = match TcpListener::bind(listen_addr) {
            Ok(x) => {
                info!("Remote debugger listening at {}", x.local_addr().unwrap());
                x
            }
            Err(e) => {
                panic!("Failed to bind listen socket for debugger: {:?}", e);
            }
        };

        let parser_shortcircuit = emu_out.clone();
        std::thread::spawn(move || {
            NetworkThread::run(listener, thread_out, parser_shortcircuit, thread_in)
        });

        Debugger {
            commands_in: emu_in,
            responses_out: emu_out,
            commands_executed: 0,
            paused: false,
        }
    }

    #[cfg(test)]
    pub fn create_for_test() -> (Self, Sender<Command>, Receiver<Response>) {
        let (cmd_tx, cmd_rx) = channel();
        let (resp_tx, resp_rx) = channel();

        (
            Self {
                commands_in: cmd_rx,
                responses_out: resp_tx,
                commands_executed: 0,
                paused: false,
            },
            cmd_tx,
            resp_rx,
        )
    }

    /// Process debugger commands, returning whether the system is currently allowed to run.
    pub fn run(&mut self, _emu: &mut Emulator, cpu: &mut Z80) -> bool {
        loop {
            let command = match self.commands_in.try_recv() {
                Ok(command) => command,
                Err(TryRecvError::Empty) => break,
                Err(e) => {
                    error!("Debugger died: {:?}", e);
                    break;
                }
            };

            info!("Core got command: {:?}", command);
            let response = match command {
                Command::Version => Response::Version(crate::built_info::PKG_VERSION.to_string()),
                Command::GetRegisters => self.read_registers(cpu),
                Command::SetRegisters(regs) => self.write_registers(cpu, regs),
                Command::Pause => {
                    self.pause();
                    Response::Ok
                }
                Command::Resume => {
                    self.unpause();
                    Response::Ok
                }
                _ => Response::NotImplemented,
            };
            if let Err(e) = self.responses_out.send(response) {
                error!("Debugger died, unable to send response: {:?}", e);
            }
            self.commands_executed = self.commands_executed.wrapping_add(1);
        }

        trace!(
            "Debugger executed {} action(s) total",
            self.commands_executed
        );
        self.paused
    }

    fn read_registers(&mut self, cpu: &Z80) -> Response {
        let regs = cpu.regs();
        Response::RegisterValues(Registers {
            af: regs.af,
            bc: regs.bc,
            de: regs.de,
            hl: regs.hl,
            ix: regs.ix,
            iy: regs.iy,
            pc: regs.pc,
            sp: regs.sp,
            i: regs.i,
            r: regs.r,
            af_: regs.af_,
            bc_: regs.bc_,
            de_: regs.de_,
            hl_: regs.hl_,
        })
    }

    fn write_registers(&mut self, cpu: &mut Z80, values: Registers) -> Response {
        let regs = cpu.regs_mut();
        regs.af = values.af;
        regs.bc = values.bc;
        regs.de = values.bc;
        regs.hl = values.bc;
        regs.ix = values.bc;
        regs.iy = values.bc;
        regs.pc = values.bc;
        regs.sp = values.bc;
        regs.i = values.i;
        regs.r = values.r;
        regs.af_ = values.af_;
        regs.bc_ = values.bc_;
        regs.de_ = values.de_;
        regs.hl_ = values.hl_;

        Response::Ok
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn unpause(&mut self) {
        self.paused = false;
    }
}

struct NetworkThread;

impl NetworkThread {
    fn run(
        listener: TcpListener,
        commands_out: Sender<Command>,
        responses_out: Sender<Response>,
        mut responses_in: Receiver<Response>,
    ) -> IoResult<()> {
        loop {
            let (socket, peer_addr) = match listener.accept() {
                Err(e) => {
                    error!("Failed to accept connection for debugging: {:?}", e);
                    return Err(e);
                }
                Ok(s) => s,
            };
            info!("Accepted remote debug connection from {}", peer_addr);

            let input_buf = {
                let rsock = socket
                    .try_clone()
                    .expect("Unable to clone debug socket for reading");
                BufReader::new(rsock)
            };
            let output_buf = BufWriter::new(socket);

            let result = Self::handle_connection(
                commands_out.clone(),
                responses_out.clone(),
                &mut responses_in,
                input_buf,
                output_buf,
            );
            info!("Debugger disconnected: {:?}", result);
        }
    }

    fn handle_connection<R: std::io::Read + Send + 'static, W: std::io::Write>(
        commands_out: Sender<Command>,
        responses_out: Sender<Response>,
        responses_in: &mut Receiver<Response>,
        input: R,
        mut output: W,
    ) -> std::io::Result<()> {
        // Command thread deserializes commands from the input and passes them to the core.
        std::thread::spawn(move || {
            let mut ct = CommandThread;
            ct.run(commands_out, responses_out, input)
        });

        // Forward responses from the core back out to the network.
        loop {
            let response = match responses_in.recv() {
                Ok(r) => r,
                Err(e) => {
                    error!("No more debug responses: {:?}", e);
                    writeln!(output, "error: unable to communicate with core")?;
                    return Ok(());
                }
            };
            debug!("Got response: {:?}", response);

            serde_json::to_writer(&mut output, &response)?;
            write!(output, "\n")?;
            output.flush()?;
        }
    }
}

struct CommandThread;

impl CommandThread {
    fn run<R: std::io::Read>(
        &mut self,
        commands: Sender<Command>,
        responses: Sender<Response>,
        input: R,
    ) {
        let mut deserializer = serde_json::Deserializer::from_reader(input);

        loop {
            debug!("Command thread waiting for command");
            let command = match Command::deserialize(&mut deserializer) {
                Ok(c) => c,
                Err(e) => {
                    use serde_json::error::Category;
                    let message = if [Category::Io, Category::Eof].contains(&e.classify()) {
                        "I/O error"
                    } else {
                        "Malformed or unrecognized command"
                    };
                    let _ = responses.send(Response::Invalid(message));
                    return;
                }
            };
            debug!("Command thread received command: {:?}", command);
            commands.send(command).expect("Command receiver hung up");
        }
    }
}
