use crate::collections::AddressSet;
use crate::{Emulator, Z80};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufWriter, Result as IoResult};
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Command {
    /// Return the provided value, [Response::SyncResponse]
    Sync(f64),

    /// Block until the CPU pauses, returning [Response::WaitResult].
    ///
    /// If the CPU is already paused, returns immediately.
    Wait,
    /// Pause the CPU, interrupting any active wait.
    ///
    /// No response is returned from this command; it is intended as an out-of-band
    /// signal to unblock the command queue, because a [Wait] will block all other
    /// commands.
    Interrupt,
    /// Resume execution, such as after a breakpoint or [Interrupt] command.
    Run,

    /// Return the value of all registers, [Response::RegisterValues]
    GetRegisters,
    /// Set the value of all registers
    SetRegisters(Registers),

    /// Add a breakpoint
    AddBreakpoint(u16),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Response {
    // Synchronized with the given value, and returns the emulator version.
    SyncResponse(f64, String),

    /// The command completed with no interesting output
    Ok,
    /// The command was invalid and has been ignored.
    Invalid(&'static str),
    /// The command is not implemented.
    NotImplemented,

    /// A Wait command has completed, with the CPU stopping for the given reason.
    WaitResult(PauseReason),
    /// The values of the CPU registers as requested.
    RegisterValues(Registers),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum PauseReason {
    AlreadyPaused,
    Terminated,
    Breakpoint,
    Interrupted,
}

#[derive(Debug)]
pub struct Debugger {
    /// Normal-priority commands, processed in order.
    commands_in: Receiver<Command>,
    /// Break requests, processed at higher priority than other commands.
    interrupts_in: Receiver<()>,
    /// Responses to all commands, in the order they are processed.
    responses_out: Sender<Response>,
    /// Counts the number of commands processed, through sending the response (if any).
    commands_executed: u32,
    /// If true, the CPU should not run.
    cpu_paused: bool,
    /// If true, the debugger is executing a wait instruction and will only process
    /// Interrupt requests.
    waiting: bool,

    breakpoints: AddressSet,
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
        let (intr_out, intr_in) = channel();
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
            NetworkThread::run(
                listener,
                thread_out,
                intr_out,
                parser_shortcircuit,
                thread_in,
            )
        });

        Debugger {
            commands_in: emu_in,
            interrupts_in: intr_in,
            responses_out: emu_out,
            commands_executed: 0,
            cpu_paused: false,
            waiting: false,
            breakpoints: Default::default(),
        }
    }

    #[cfg(test)]
    pub fn create_for_test() -> (Self, Sender<Command>, Receiver<Response>) {
        let (cmd_tx, cmd_rx) = channel();
        let (_intr_tx, intr_rx) = channel();
        let (resp_tx, resp_rx) = channel();

        (
            Self {
                commands_in: cmd_rx,
                interrupts_in: intr_rx,
                responses_out: resp_tx,
                commands_executed: 0,
                cpu_paused: false,
                waiting: false,
                breakpoints: Default::default(),
            },
            cmd_tx,
            resp_rx,
        )
    }

    pub fn is_paused(&self) -> bool {
        self.cpu_paused
    }

    fn send_response(&mut self, response: Response) {
        if let Err(e) = self.responses_out.send(response) {
            error!("Debugger died, unable to send response: {:?}", e);
        }
        self.record_command_completion();
    }

    fn record_command_completion(&mut self) {
        self.commands_executed = self.commands_executed.wrapping_add(1);
    }

    /// Clear any active wait, sending its response and pausing the CPU.
    fn finish_wait(&mut self, reason: PauseReason) {
        if self.waiting {
            self.waiting = false;
            self.send_response(Response::WaitResult(reason));
        }
        self.cpu_paused = true;
    }

    /// Process debugger commands, returning whether the system is currently allowed to run.
    pub fn run(&mut self, emu: &mut Emulator, cpu: &mut Z80) -> bool {
        // If emulation has terminated and we're waiting, stop waiting.
        if !emu.is_running() && self.waiting {
            self.finish_wait(PauseReason::Terminated);
        }

        // Handle interrupts, which pause emulation until resumed
        match self.interrupts_in.try_recv() {
            Ok(_) => {
                self.finish_wait(PauseReason::Interrupted);
                // Interrupt doesn't send a response, so manually record completion.
                self.record_command_completion();
            }
            Err(TryRecvError::Empty) => {}
            Err(e) => {
                error!("Interrupt sender died: {:?}", e);
            }
        }

        while !self.waiting {
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
                Command::Sync(token) => {
                    Response::SyncResponse(token, crate::built_info::PKG_VERSION.to_string())
                }
                Command::GetRegisters => self.read_registers(cpu),
                Command::SetRegisters(regs) => self.write_registers(cpu, regs),
                Command::Run => {
                    self.unpause();
                    Response::Ok
                }
                Command::Wait => {
                    debug_assert!(!self.waiting, "Reentrant waits don't make sense");
                    if !self.cpu_paused {
                        // Not already paused: stop processing commands until the wait completes.
                        self.waiting = true;
                        break;
                    }
                    // Already paused, say so.
                    Response::WaitResult(PauseReason::AlreadyPaused)
                }
                Command::AddBreakpoint(addr) => {
                    self.breakpoints.insert(addr);
                    Response::Ok
                }
                _ => Response::NotImplemented,
            };
            self.send_response(response);
        }

        trace!(
            "Debugger executed {} action(s) total",
            self.commands_executed
        );
        self.cpu_paused
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
        self.finish_wait(PauseReason::Interrupted);
    }

    pub fn unpause(&mut self) {
        self.cpu_paused = false;
    }

    /// Test for a breakpoint hit and return whether the CPU should stop.
    ///
    /// Also handles any debugger activity in response to hitting a breakpoint.
    pub fn handle_instruction_fetch(&mut self, addr: u16) -> bool {
        let hit = self.breakpoints.contains(&addr);
        if hit {
            self.finish_wait(PauseReason::Breakpoint);
        }
        hit
    }
}

struct NetworkThread;

impl NetworkThread {
    fn run(
        listener: TcpListener,
        commands_out: Sender<Command>,
        interrupts_out: Sender<()>,
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
                interrupts_out.clone(),
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
        interrupts_out: Sender<()>,
        responses_out: Sender<Response>,
        responses_in: &mut Receiver<Response>,
        input: R,
        mut output: W,
    ) -> std::io::Result<()> {
        // Command thread deserializes commands from the input and passes them to the core.
        std::thread::spawn(move || {
            let mut ct = CommandThread;
            debug!("Command thread starting");
            ct.run(commands_out, interrupts_out, responses_out, input);
            debug!("Command thread terminating");
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

            // Disconnect on protocol error, which may also include the client disconnecting.
            if let Response::Invalid(_) = response {
                return Ok(());
            }
        }
    }
}

struct CommandThread;

impl CommandThread {
    fn run<R: std::io::Read>(
        &mut self,
        commands: Sender<Command>,
        interrupts: Sender<()>,
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
                    let message = if e.classify() == Category::Io {
                        "I/O error"
                    } else if e.classify() == Category::Eof {
                        // Generate a response so handle_connection attempts to write
                        // output and drops the connection.
                        "Client disconnected"
                    } else {
                        "Malformed or unrecognized command"
                    };
                    let _ = responses.send(Response::Invalid(message));
                    return;
                }
            };
            debug!("Command thread received command: {:?}", command);

            if command == Command::Interrupt {
                interrupts.send(()).expect("Interrupt receiver hung up");
            } else {
                commands.send(command).expect("Command receiver hung up");
            }
        }
    }
}
