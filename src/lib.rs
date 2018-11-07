#![warn(missing_docs)]
//! Library to load, parse, validate, and iterate over [Bufkit][1] files.
//!
//! [1]: http://www.wdtb.noaa.gov/tools/BUFKIT/

//
// API
//

pub use bufkit_data::{BufkitData, BufkitFile, SoundingIterator};
pub use error::*;

//
// Internal use only
//
extern crate chrono;
extern crate optional;

extern crate metfor;
extern crate sounding_analysis;
extern crate sounding_base;

mod bufkit_data;
mod error;
mod parse_util;
