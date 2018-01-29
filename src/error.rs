//! Errors specific to the sounding-bufkit crate.
use std::fmt::{Display, Formatter, Result};

use failure::{Fail, Backtrace};

pub use failure::Error;

/// Basic error originating in this crate with a backtrace.
#[derive(Debug)]
pub struct BufkitFileError {
    backtrace: Backtrace,
}

impl BufkitFileError {
    /// Createa new BufkitFileError.
    pub fn new() -> BufkitFileError{
        BufkitFileError{backtrace: Backtrace::new() }
    }
}

impl Display for BufkitFileError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        writeln!(f, "Error parsing bufkit file.")?;
        write!(f, "{}", self.backtrace)
    }
}

impl Fail for BufkitFileError {
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(&self.backtrace)
    }
}

impl Default for BufkitFileError {
    fn default()->Self {
        BufkitFileError::new()
    }
}
