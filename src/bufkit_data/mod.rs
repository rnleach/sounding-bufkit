//! Module for reading a bufkit file and breaking it into smaller pieces for parsing later.

use std::path::Path;

mod upper_air_section;
mod surface_section;
mod upper_air;
mod surface;

use sounding_base::{Sounding, StationInfo};

use self::surface::SurfaceData;
use self::surface_section::{SurfaceIterator, SurfaceSection};
use self::upper_air::UpperAir;
use self::upper_air_section::{UpperAirIterator, UpperAirSection};
use error::*;

/// Hold an entire bufkit file in memory.
pub struct BufkitFile {
    file_text: String,
}

impl BufkitFile {
    /// Load a file into memory.
    pub fn load(path: &Path) -> Result<BufkitFile, Error> {
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::Read;

        // Load the file contents
        let mut file = BufReader::new(File::open(path)?);
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(BufkitFile {
            file_text: contents,
        })
    }

    /// Validate the whole file, ensure it is parseable and do some sanity checks.
    pub fn validate_file_format(&self) -> Result<(), Error> {
        let data = self.data()?;
        data.validate()?;

        Ok(())
    }

    /// Get a bufkit data object from this file.
    pub fn data(&self) -> Result<BufkitData, Error> {
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
    pub fn validate(&self) -> Result<(), Error> {
        self.upper_air.validate_section()?;
        self.surface.validate_section()?;
        Ok(())
    }

    /// Create a new data representation from a string
    pub fn new(text: &str) -> Result<BufkitData, Error> {
        let break_point = BufkitData::find_break_point(text)?;
        let data = BufkitData::new_with_break_point(text, break_point)?;
        Ok(data)
    }

    fn new_with_break_point(text: &str, break_point: usize) -> Result<BufkitData, BufkitFileError> {
        Ok(BufkitData {
            upper_air: UpperAirSection::new(&text[0..break_point]),
            surface: SurfaceSection::new(&text[break_point..])?,
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
    type Item = Sounding;
    type IntoIter = SoundingIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SoundingIterator {
            upper_air_it: self.upper_air.into_iter(),
            surface_it: self.surface.into_iter(),
        }
    }
}

fn combine_data(ua: &UpperAir, sd: &SurfaceData) -> Sounding {
    use sounding_base::Profile;
    use sounding_base::Index;
    use sounding_base::Surface;

    // Missing or no data values used in Bufkit files
    const MISSING_I32: i32 = -9999;
    const MISSING_F64: f64 = -9999.0;

    #[inline(always)]
    fn check_missing(val: f64) -> Option<f64> {
        if val == MISSING_F64 { None } else { Some(val)}
    }

    #[inline(always)]
    fn check_missing_i32(val: i32) -> Option<i32> {
        if val == MISSING_I32 { None } else { Some(val)}
    }

    let coords = check_missing(ua.lat).and_then(|lat| check_missing(ua.lon).and_then(|lon| Some((lat,lon))));
    let station = StationInfo::new_with_values(check_missing_i32(ua.num), coords, check_missing(ua.elevation));

    Sounding::new()
        .set_station_info(station)
        .set_valid_time(ua.valid_time)
        .set_lead_time(check_missing_i32(ua.lead_time))

        // Indexes
        .set_index(Index::Showalter,check_missing(ua.show))
        .set_index(Index::LI, check_missing(ua.li))
        .set_index(Index::SWeT, check_missing(ua.swet))
        .set_index(Index::K, check_missing(ua.kinx))
        .set_index(Index::LCL, check_missing(ua.lclp))
        .set_index(Index::PWAT, check_missing(ua.pwat))
        .set_index(Index::TotalTotals, check_missing(ua.totl))
        .set_index(Index::CAPE, check_missing(ua.cape))
        .set_index(Index::LCLTemperature, check_missing(ua.lclt))
        .set_index(Index::CIN, check_missing(ua.cins))
        .set_index(Index::EquilibrimLevel, check_missing(ua.eqlv))
        .set_index(Index::LFC, check_missing(ua.lfc))
        .set_index(Index::BulkRichardsonNumber, check_missing(ua.brch))

        // Upper air
        .set_profile(Profile::Pressure,
            ua.pressure.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(Profile::Temperature,
            ua.temperature.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(Profile::WetBulb,
            ua.wet_bulb.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(Profile::DewPoint,
            ua.dew_point.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(Profile::ThetaE,
            ua.theta_e.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(Profile::WindDirection,
            ua.direction.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(Profile::WindSpeed,
            ua.speed.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(Profile::PressureVerticalVelocity,
            ua.omega.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(Profile::GeopotentialHeight,
            ua.height.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(Profile::CloudFraction,
            ua.cloud_fraction.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())

        // Surface data
        .set_surface_value(Surface::MSLP, check_missing(sd.mslp))
        .set_surface_value(Surface::StationPressure, check_missing(sd.station_pres))
        .set_surface_value(Surface::LowCloud, check_missing(sd.low_cloud))
        .set_surface_value(Surface::MidCloud, check_missing(sd.mid_cloud))
        .set_surface_value(Surface::HighCloud, check_missing(sd.hi_cloud))
        .set_surface_value(Surface::UWind, check_missing(sd.uwind))
        .set_surface_value(Surface::VWind, check_missing(sd.vwind))
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
                return Some(combine_data(&next_ua, &next_sd));
            }
        }
    }
}
