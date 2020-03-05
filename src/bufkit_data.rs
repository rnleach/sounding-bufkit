//! Module for reading a bufkit file and breaking it into smaller pieces for parsing later.
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

mod combine;
mod surface;
mod surface_section;
mod upper_air;
mod upper_air_section;

use sounding_analysis::Sounding;

use self::surface_section::{SurfaceIterator, SurfaceSection};
use self::upper_air_section::{UpperAirIterator, UpperAirSection};
use crate::error::*;

/// Hold an entire bufkit file in memory.
pub struct BufkitFile {
    file_text: String,
    file_name: String,
}

impl BufkitFile {
    /// Load a file into memory.
    pub fn load(path: &Path) -> Result<BufkitFile, Box<dyn Error>> {
        use std::fs::File;
        use std::io::prelude::Read;
        use std::io::BufReader;

        // Load the file contents
        let mut file = BufReader::new(File::open(path)?);
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(BufkitFile {
            file_text: contents,
            file_name: path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown File".to_owned()),
        })
    }

    /// Validate the whole file, ensure it is parseable and do some sanity checks.
    pub fn validate_file_format(&self) -> Result<(), Box<dyn Error>> {
        let data = self.data()?;
        data.validate()?;

        Ok(())
    }

    /// Get a bufkit data object from this file.
    pub fn data(&self) -> Result<BufkitData<'_>, Box<dyn Error>> {
        BufkitData::init(&self.file_text, &self.file_name)
    }

    /// Get the raw string data from the file.
    pub fn raw_text(&self) -> &str {
        &self.file_text
    }
}

/// References to different data sections within a `BufkitFile` mainly useful for generating
/// iterators.
///
/// This is theoretically not necessary without lexical lifetimes.
pub struct BufkitData<'a> {
    upper_air: UpperAirSection<'a>,
    surface: SurfaceSection<'a>,
    file_name: &'a str,
}

impl<'a> BufkitData<'a> {
    /// Validate the whole string, ensure it is parseable and do some sanity checks.
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        self.upper_air.validate_section()?;
        self.surface.validate_section()?;
        Ok(())
    }

    /// Initialize struct for parsing a sounding.
    pub fn init(text: &'a str, fname: &'a str) -> Result<BufkitData<'a>, Box<dyn Error>> {
        let break_point = BufkitData::find_break_point(text)?;
        let data = BufkitData::new_with_break_point(text, break_point, fname)?;
        Ok(data)
    }

    fn new_with_break_point(
        text: &'a str,
        break_point: usize,
        fname: &'a str,
    ) -> Result<BufkitData<'a>, BufkitFileError> {
        Ok(BufkitData {
            upper_air: UpperAirSection::new(&text[0..break_point]),
            surface: SurfaceSection::init(&text[break_point..])?,
            file_name: fname,
        })
    }

    fn find_break_point(text: &str) -> Result<usize, BufkitFileError> {
        match text.find("STN YYMMDD/HHMM") {
            None => Err(BufkitFileError::new()),
            Some(val) => Ok(val),
        }
    }
}

impl<'a> IntoIterator for &'a BufkitData<'a> {
    type Item = (Sounding, HashMap<&'static str, f64>);
    type IntoIter = SoundingIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SoundingIterator {
            upper_air_it: self.upper_air.into_iter(),
            surface_it: self.surface.into_iter(),
            source_name: self.file_name,
        }
    }
}

/// Iterator type for `BufkitData` that returns a `Sounding`.
pub struct SoundingIterator<'a> {
    upper_air_it: UpperAirIterator<'a>,
    surface_it: SurfaceIterator<'a>,
    source_name: &'a str,
}

impl<'a> Iterator for SoundingIterator<'a> {
    type Item = (Sounding, HashMap<&'static str, f64>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_ua = self.upper_air_it.next()?;
        let mut next_sd = self.surface_it.next()?;

        loop {
            while next_sd.valid_time < next_ua.valid_time {
                next_sd = self.surface_it.next()?;
            }
            while next_ua.valid_time < next_sd.valid_time {
                next_ua = self.upper_air_it.next()?;
            }
            if next_ua.valid_time == next_sd.valid_time {
                return Some(combine::combine_data(next_ua, next_sd, &self.source_name));
            }
        }
    }
}
