use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

trait DebugInterface {}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Command {
    /// Pause execution until resumed.
    Pause,
    /// Resume execution, such as after a breakpoint or pause command
    Resume,
    /// Return the emulator version, [Response::Version]
    Version,
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
}

#[cfg(feature = "remote-debug")]
mod remote;

#[cfg(feature = "remote-debug")]
pub use remote::RemoteDebugger;

#[derive(Debug, Default)]
pub struct DummyDebugger;

impl DebugInterface for DummyDebugger {}
