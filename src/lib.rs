#![warn(missing_docs)]
//! Library to load, parse, validate, and iterate over [Bufkit][1] files.
//!
//! [1]: http://www.wdtb.noaa.gov/tools/BUFKIT/
#![recursion_limit = "1024"]
extern crate chrono;
#[macro_use]
extern crate error_chain;

extern crate sounding_base;

mod bufkit_data;
mod error;
mod parse_util;

pub use error::*;
pub use bufkit_data::{BufkitFile, BufkitData, SoundingIterator};
