use custom_error::custom_error;
use serde::Serialize;
use serde::ser::Serializer;
use serde_json::{json, to_vec};
use std::collections::HashMap;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::Path;

custom_error! {pub LCDError
    DriverError{comment:&'static str} = "{comment}"
}

#[derive(Debug)]
pub struct LCDdriver {
    driver_stream: UnixStream,
}

#[derive(Serialize)]
pub enum LCDProgramm {
    Clear,
    Move,
    Bcklight,
    CursorMode,
    ShiftDisplay,
    Home,
    Write,
}

pub enum LCDArg {
    String(String),
    Int(i128),
    Bool(bool)
}

impl Serialize for LCDArg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            LCDArg::String(ref s) => serializer.serialize_str(s),
            LCDArg::Int(i) => serializer.serialize_i128(*i),
            LCDArg::Bool(b) => serializer.serialize_bool(*b),
        }
    }
}

#[derive(Serialize)]
pub struct LCDCommand {
    pub cmd: LCDProgramm,
    pub args: Option<HashMap<String, LCDArg>>,
}

impl LCDdriver {
    pub fn new(socket_path: &Path, clear: bool) -> Result<LCDdriver, LCDError> {
        let mut driver = LCDdriver {
            driver_stream: UnixStream::connect(socket_path)
                .map_err(|_| LCDError::DriverError { comment: "Could not construct driver!" })?,
        };

        if clear {
            driver.exec(LCDCommand {
                cmd: LCDProgramm::Clear,
                args: None,
            })?;
            driver.exec(LCDCommand {
                cmd: LCDProgramm::Home,
                args: None,
            })?;
        }
        Ok(driver)
    }

    pub fn exec(&mut self, command: LCDCommand) -> Result<(), LCDError> {
        let mut json_command = to_vec(&json!(command))
            .map_err(|_| LCDError::DriverError { comment: "Serialization failed" })?;
        json_command.push('\n' as u8);
        self.driver_stream
            .write_all(&json_command)
            .map_err(|e| {let msg = format!("Could not write to socket {:?}", e.to_string()); return LCDError::DriverError { comment: Box::leak(msg.into_boxed_str()) }})
    }
}
