#![warn(missing_docs)]
//! Library to load, parse, validate, and iterate over [Bufkit][1] files.
//!
//! [1]: http://www.wdtb.noaa.gov/tools/BUFKIT/
extern crate chrono;
extern crate failure;

extern crate sounding_base;

mod bufkit_data;
mod error;
mod parse_util;

pub use error::*;
pub use bufkit_data::{BufkitData, BufkitFile, SoundingIterator};
