// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;

use chrono::Utc;
use rand::random;
use serde::{Deserialize, Serialize};

pub mod collector;
pub mod reporter;

/// Reporter id is a unique identifier for a reporter.
///
/// It is used to identify the process that sends the execution report.
/// Because the OS PID is not unique across a single build (PIDs are
/// recycled), we need to use a new unique identifier to identify the process.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ReporterId(pub u64);

impl ReporterId {
    pub fn new() -> Self {
        let id = random::<u64>();
        ReporterId(id)
    }
}

/// Provides a default implementation for `ReporterId`.
///
/// This implementation returns a new instance of `ReporterId` by calling the `new` method.
///
/// # Examples
///
/// ```
/// use intercept::ReporterId;
///
/// let default_reporter_id = ReporterId::default();
/// ```
impl Default for ReporterId {
    fn default() -> Self {
        Self::new()
    }
}

/// Process id is a OS identifier for a process.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProcessId(pub u32);

/// Execution is a representation of a process execution.
///
/// It does not contain information about the outcome of the execution,
/// like the exit code or the duration of the execution. It only contains
/// the information that is necessary to reproduce the execution.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Execution {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub working_dir: PathBuf,
    pub environment: HashMap<String, String>,
}

/// Represent a relevant life cycle event of a process.
///
/// In the current implementation, we only have one event, the `Started` event.
/// This event is sent when a process is started. It contains the process id
/// and the execution information.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Event {
    pub pid: ProcessId,
    pub execution: Execution,
}

/// Envelope is a wrapper around the event.
///
/// It contains the reporter id, the timestamp of the event and the event itself.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Envelope {
    pub rid: ReporterId,
    pub timestamp: u64,
    pub event: Event,
}

impl Envelope {
    pub fn new(rid: &ReporterId, event: Event) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u64;
        Envelope {
            rid: rid.clone(),
            timestamp,
            event,
        }
    }

    /// Read an envelope from a reader using TLV format.
    ///
    /// The envelope is serialized using JSON and the length of the JSON
    /// is written as a 4 byte big-endian integer before the JSON.
    pub fn read_from(reader: &mut impl Read) -> Result<Self, anyhow::Error> {
        let mut length_bytes = [0; 4];
        reader.read_exact(&mut length_bytes)?;
        let length = u32::from_be_bytes(length_bytes) as usize;

        let mut buffer = vec![0; length];
        reader.read_exact(&mut buffer)?;
        let envelope = serde_json::from_slice(buffer.as_ref())?;

        Ok(envelope)
    }

    /// Write an envelope to a writer using TLV format.
    ///
    /// The envelope is serialized using JSON and the length of the JSON
    /// is written as a 4 byte big-endian integer before the JSON.
    pub fn write_into(&self, writer: &mut impl Write) -> Result<u32, anyhow::Error> {
        let serialized_envelope = serde_json::to_string(&self)?;
        let bytes = serialized_envelope.into_bytes();
        let length = bytes.len() as u32;

        writer.write_all(&length.to_be_bytes())?;
        writer.write_all(&bytes)?;

        Ok(length)
    }
}

/// Declare the environment variable name for the reporter address.
pub const KEY_DESTINATION: &str = "INTERCEPT_REPORTER_ADDRESS";
