//! Errors specific to the sounding-bufkit crate.
use std::error::Error;
use std::fmt::{Display, Formatter, Result};

/// Basic error originating in this crate with a backtrace.
#[derive(Debug)]
pub struct BufkitFileError {}

impl BufkitFileError {
    /// Createa new BufkitFileError.
    pub fn new() -> BufkitFileError {
        BufkitFileError {}
    }
}

impl Display for BufkitFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Error parsing bufkit file.")
    }
}

impl Error for BufkitFileError {}

impl Default for BufkitFileError {
    fn default() -> Self {
        BufkitFileError::new()
    }
}
