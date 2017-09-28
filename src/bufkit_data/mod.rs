//! Module for reading a bufkit file and breaking it into smaller pieces for parsing later.

use std::path::Path;

mod upper_air_section;
mod surface_section;
mod upper_air;
mod surface;

use sounding_base::{Sounding, MissingData};

use self::surface::SurfaceData;
use self::surface_section::{SurfaceSection, SurfaceIterator};
use self::upper_air::UpperAir;
use self::upper_air_section::{UpperAirSection, UpperAirIterator};
use error::*;

/// Hold an entire bufkit file in memory.
pub struct BufkitFile {
    file_text: String,
}

impl BufkitFile {
    /// Load a file into memory.
    pub fn load(path: &Path) -> Result<BufkitFile> {
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::Read;

        // Load the file contents
        let mut file = BufReader::new(File::open(path).chain_err(|| "Unable to opend file.")?);
        let mut contents = String::new();
        file.read_to_string(&mut contents).chain_err(
            || "Unable to read file.",
        )?;

        Ok(BufkitFile { file_text: contents })
    }

    /// Validate the whole file, ensure it is parseable and do some sanity checks.
    pub fn validate(&self) -> Result<()> {
        let data = self.data().chain_err(
            || "Unable to split upper air and surface sections.",
        )?;
        data.validate().chain_err(|| "Failed validation.")?;
        Ok(())
    }

    /// Get a bufkit data object from this file.
    pub fn data<'a>(&'a self) -> Result<BufkitData<'a>> {
        BufkitData::new(&self.file_text)
    }
}

/// References to different data sections within a `BufkitFile` mainly useful for generating
/// iterators.
///
/// This is theoretically not necessary without lexical lifetimes.
pub struct BufkitData<'a> {
    upper_air: UpperAirSection<'a>,
    surface: SurfaceSection<'a>,
}

impl<'a> BufkitData<'a> {
    /// Validate the whole string, ensure it is parseable and do some sanity checks.
    pub fn validate(&self) -> Result<()> {
        self.upper_air.validate_section().chain_err(
            || "Failed upper air section.",
        )?;
        self.surface.validate_section().chain_err(
            || "Failed surface section.",
        )?;
        Ok(())
    }

    /// Create a new data representation from a string
    pub fn new<'b>(text: &'b str) -> Result<BufkitData<'b>> {
        let break_point = BufkitData::find_break_point(text)?;
        BufkitData::new_with_break_point(text, break_point)
    }

    fn new_with_break_point<'b>(text: &'b str, break_point: usize) -> Result<BufkitData<'b>> {
        Ok(BufkitData {
            upper_air: UpperAirSection::new(&text[0..break_point]),
            surface: SurfaceSection::new(&text[break_point..]).chain_err(
                || "Unable to get surface section.",
            )?,
        })
    }

    fn find_break_point(text: &str) -> Result<usize> {
        match text.find("STN YYMMDD/HHMM") {
            None => Err(Error::from(
                "Unable to find break between surface and upper air data.",
            )),
            Some(val) => Ok(val),
        }
    }
}

impl<'a> IntoIterator for &'a BufkitData<'a> {
    type Item = Sounding;
    type IntoIter = SoundingIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SoundingIterator {
            upper_air_it: self.upper_air.into_iter(),
            surface_it: self.surface.into_iter(),
        }
    }
}

fn combine_data(ua: UpperAir, sd: SurfaceData) -> Sounding {

    Sounding {
        // Station info
        num: ua.num.into(),
        valid_time: ua.valid_time,
        lead_time: ua.lead_time.into(),
        lat: ua.lat.into(),
        lon: ua.lon.into(),
        elevation: ua.elevation.into(),

        // Indexes
        show: ua.show.into(),
        li: ua.li.into(),
        swet: ua.swet.into(),
        kinx: ua.kinx.into(),
        lclp: ua.lclp.into(),
        pwat: ua.pwat.into(),
        totl: ua.totl.into(),
        cape: ua.cape.into(),
        lclt: ua.lclt.into(),
        cins: ua.cins.into(),
        eqlv: ua.eqlv.into(),
        lfc: ua.lfc.into(),
        brch: ua.brch.into(),
        hain: i32::MISSING.into(),

        // Upper air
        pressure: ua.pressure,
        temperature: ua.temperature,
        wet_bulb: ua.wet_bulb,
        dew_point: ua.dew_point,
        theta_e: ua.theta_e,
        direction: ua.direction,
        speed: ua.speed,
        omega: ua.omega,
        height: ua.height,
        cloud_fraction: ua.cloud_fraction,

        // Surface data
        mslp: sd.mslp.into(),
        station_pres: sd.station_pres.into(),
        low_cloud: sd.low_cloud.into(),
        mid_cloud: sd.mid_cloud.into(),
        hi_cloud: sd.hi_cloud.into(),
        uwind: sd.uwind.into(),
        vwind: sd.vwind.into(),
    }
}

/// Iterator type for `BufkitData` that returns a `Sounding`.
pub struct SoundingIterator<'a> {
    upper_air_it: UpperAirIterator<'a>,
    surface_it: SurfaceIterator<'a>,
}

impl<'a> Iterator for SoundingIterator<'a> {
    type Item = Sounding;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_ua_opt = self.upper_air_it.next();
        if next_ua_opt.is_none() {
            return None;
        }
        let mut next_ua = next_ua_opt.unwrap();

        let mut next_sd_opt = self.surface_it.next();
        if next_sd_opt.is_none() {
            return None;
        }
        let mut next_sd = next_sd_opt.unwrap();

        loop {
            while next_sd.valid_time < next_ua.valid_time {
                next_sd_opt = self.surface_it.next();
                if next_sd_opt.is_none() {
                    return None;
                }
                next_sd = next_sd_opt.unwrap();
            }
            while next_ua.valid_time < next_sd.valid_time {
                next_ua_opt = self.upper_air_it.next();
                if next_ua_opt.is_none() {
                    return None;
                }
                next_ua = next_ua_opt.unwrap();
            }
            if next_ua.valid_time == next_sd.valid_time {
                return Some(combine_data(next_ua, next_sd));
            }
        }
    }
}
