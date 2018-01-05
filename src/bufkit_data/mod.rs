//! Module for reading a bufkit file and breaking it into smaller pieces for parsing later.

use std::path::Path;

mod upper_air_section;
mod surface_section;
mod upper_air;
mod surface;

use sounding_base::Sounding;

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
    pub fn load(path: &Path) -> Result<BufkitFile> {
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::Read;

        // Load the file contents
        let mut file = BufReader::new(File::open(path).chain_err(|| "Unable to opend file.")?);
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .chain_err(|| "Unable to read file.")?;

        Ok(BufkitFile {
            file_text: contents,
        })
    }

    /// Validate the whole file, ensure it is parseable and do some sanity checks.
    pub fn validate_file_format(&self) -> Result<()> {
        let data = self.data()
            .chain_err(|| "Unable to split upper air and surface sections.")?;
        data.validate()
            .chain_err(|| "Failed validation of file format.")?;

        Ok(())
    }

    /// Get a bufkit data object from this file.
    pub fn data(&self) -> Result<BufkitData> {
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
        self.upper_air
            .validate_section()
            .chain_err(|| "Failed upper air section.")?;
        self.surface
            .validate_section()
            .chain_err(|| "Failed surface section.")?;
        Ok(())
    }

    /// Create a new data representation from a string
    pub fn new(text: &str) -> Result<BufkitData> {
        let break_point = BufkitData::find_break_point(text)?;
        BufkitData::new_with_break_point(text, break_point)
    }

    fn new_with_break_point(text: &str, break_point: usize) -> Result<BufkitData> {
        Ok(BufkitData {
            upper_air: UpperAirSection::new(&text[0..break_point]),
            surface: SurfaceSection::new(&text[break_point..])
                .chain_err(|| "Unable to get surface section.")?,
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

fn combine_data(ua: &UpperAir, sd: &SurfaceData) -> Sounding {
    use sounding_base::Profile::*;
    use sounding_base::Index::*;
    use sounding_base::Surface::*;

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

    Sounding::new()
        .set_station_num(check_missing_i32(ua.num))
        .set_valid_time(ua.valid_time)
        .set_lead_time(check_missing_i32(ua.lead_time))
        .set_location(check_missing(ua.lat), check_missing(ua.lon), check_missing(ua.elevation))

        // Indexes
        .set_index(Showalter,check_missing(ua.show))
        .set_index(LI, check_missing(ua.li))
        .set_index(SWeT, check_missing(ua.swet))
        .set_index(K, check_missing(ua.kinx))
        .set_index(LCL, check_missing(ua.lclp))
        .set_index(PWAT, check_missing(ua.pwat))
        .set_index(TotalTotals, check_missing(ua.totl))
        .set_index(CAPE, check_missing(ua.cape))
        .set_index(LCLTemperature, check_missing(ua.lclt))
        .set_index(CIN, check_missing(ua.cins))
        .set_index(EquilibrimLevel, check_missing(ua.eqlv))
        .set_index(LFC, check_missing(ua.lfc))
        .set_index(BulkRichardsonNumber, check_missing(ua.brch))

        // Upper air
        .set_profile(Pressure,
            ua.pressure.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(Temperature,
            ua.temperature.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(WetBulb,
            ua.wet_bulb.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(DewPoint,
            ua.dew_point.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(ThetaE,
            ua.theta_e.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(WindDirection,
            ua.direction.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(WindSpeed,
            ua.speed.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(PressureVerticalVelocity,
            ua.omega.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(GeopotentialHeight,
            ua.height.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())
        .set_profile(CloudFraction,
            ua.cloud_fraction.iter().map(|val| check_missing(*val)).collect::<Vec<_>>())

        // Surface data
        .set_surface_value(MSLP, check_missing(sd.mslp))
        .set_surface_value(StationPressure, check_missing(sd.station_pres))
        .set_surface_value(LowCloud, check_missing(sd.low_cloud))
        .set_surface_value(MidCloud, check_missing(sd.mid_cloud))
        .set_surface_value(HighCloud, check_missing(sd.hi_cloud))
        .set_surface_value(UWind, check_missing(sd.uwind))
        .set_surface_value(VWind, check_missing(sd.vwind))
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
