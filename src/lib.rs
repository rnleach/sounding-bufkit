#![warn(missing_docs)]
//! Library to load, parse, validate, and iterate over [Bufkit][1] files.
//!
//! [1]: http://www.wdtb.noaa.gov/tools/BUFKIT/

//
// API
//

pub use crate::bufkit_data::{BufkitData, BufkitFile, SoundingIterator};
pub use crate::error::*;

//
// Internal use only
//

mod bufkit_data;
mod error;
mod parse_util;
