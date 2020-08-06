use crate::debug::{Command, Response};
use serde::Deserialize;
use std::io::{BufRead, BufReader, BufWriter, Result as IoResult, Write};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

#[derive(Debug)]
pub struct RemoteDebugger {
    commands_in: Receiver<Command>,
    responses_out: Sender<Response>,
}

impl std::default::Default for RemoteDebugger {
    fn default() -> Self {
        let (thread_out, emu_in) = channel();
        let (emu_out, thread_in) = channel();

        let listener = match TcpListener::bind(("localhost", 10000)) {
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

        RemoteDebugger {
            commands_in: emu_in,
            responses_out: emu_out,
        }
    }
}

impl RemoteDebugger {
    /// Process debugger commands, returning the number that were processed.
    pub fn run(&mut self) -> usize {
        let mut n = 0;

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
                _ => Response::NotImplemented,
            };
            if let Err(e) = self.responses_out.send(response) {
                error!("Debugger died, unable to send response: {:?}", e);
            }
            n += 1;
        }

        debug!("Debugger executed {} actions in this iteration", n);
        n
    }
}

struct NetworkThread;

impl NetworkThread {
    fn run(
        listener: TcpListener,
        commands_out: Sender<Command>,
        responses_out: Sender<Response>,
        responses_in: Receiver<Response>,
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

            let commands_out_ = commands_out.clone();
            let input_buf = {
                let rsock = socket
                    .try_clone()
                    .expect("Unable to clone debug socket for reading");
                BufReader::new(rsock)
            };

            // Read commands from the network and forward to the core.
            let responses_out_ = responses_out.clone();
            std::thread::spawn(move || {
                let mut ct = CommandThread;
                ct.run(commands_out_, responses_out_, input_buf)
            });

            // Forward responses from the core back out to the network.
            let mut socket = BufWriter::new(socket);
            loop {
                let response = match responses_in.recv() {
                    Ok(r) => r,
                    Err(e) => {
                        error!("No more debug responses: {:?}", e);
                        writeln!(socket, "error: unable to communicate with core")?;
                        continue;
                    }
                };
                debug!("Got response: {:?}", response);

                serde_json::to_writer(&mut socket, &response)?;
                write!(socket, "\n")?;
                socket.flush()?;
            }
        }
    }
}

struct CommandThread;

impl CommandThread {
    fn run<R: BufRead>(
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
