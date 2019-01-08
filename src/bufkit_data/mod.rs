//! Module for reading a bufkit file and breaking it into smaller pieces for parsing later.
use crate::parse_util::check_missing_i32;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

mod surface;
mod surface_section;
mod upper_air;
mod upper_air_section;

use metfor::{MetersPSec, Quantity, WindUV};
use sounding_analysis::Analysis;
use sounding_base::{Sounding, StationInfo};

use self::surface::SurfaceData;
use self::surface_section::{SurfaceIterator, SurfaceSection};
use self::upper_air::UpperAir;
use self::upper_air_section::{UpperAirIterator, UpperAirSection};
use crate::error::*;

/// Hold an entire bufkit file in memory.
pub struct BufkitFile {
    file_text: String,
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
        })
    }

    /// Validate the whole file, ensure it is parseable and do some sanity checks.
    pub fn validate_file_format(&self) -> Result<(), Box<dyn Error>> {
        let data = self.data()?;
        data.validate()?;

        Ok(())
    }

    /// Get a bufkit data object from this file.
    pub fn data(&self) -> Result<BufkitData, Box<dyn Error>> {
        BufkitData::init(&self.file_text)
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
}

impl<'a> BufkitData<'a> {
    /// Validate the whole string, ensure it is parseable and do some sanity checks.
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        self.upper_air.validate_section()?;
        self.surface.validate_section()?;
        Ok(())
    }

    /// Initialize struct for parsing a sounding.
    pub fn init(text: &str) -> Result<BufkitData, Box<dyn Error>> {
        let break_point = BufkitData::find_break_point(text)?;
        let data = BufkitData::new_with_break_point(text, break_point)?;
        Ok(data)
    }

    fn new_with_break_point(text: &str, break_point: usize) -> Result<BufkitData, BufkitFileError> {
        Ok(BufkitData {
            upper_air: UpperAirSection::new(&text[0..break_point]),
            surface: SurfaceSection::init(&text[break_point..])?,
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
    type Item = Analysis;
    type IntoIter = SoundingIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SoundingIterator {
            upper_air_it: self.upper_air.into_iter(),
            surface_it: self.surface.into_iter(),
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn combine_data(ua: UpperAir, sd: SurfaceData) -> Analysis {
    let coords: Option<(f64, f64)> = ua
        .lat
        .into_option()
        .and_then(|lat| ua.lon.into_option().map(|lon| (lat, lon)));

    let station = StationInfo::new_with_values(check_missing_i32(ua.num), coords, ua.elevation);

    let snd = Sounding::new()
        .with_station_info(station)
        .with_valid_time(ua.valid_time)
        .with_lead_time(check_missing_i32(ua.lead_time))
        // Upper air
        .with_pressure_profile(ua.pressure)
        .with_temperature_profile(ua.temperature)
        .with_wet_bulb_profile(ua.wet_bulb)
        .with_dew_point_profile(ua.dew_point)
        .with_theta_e_profile(ua.theta_e)
        .with_wind_profile(ua.wind)
        .with_pvv_profile(ua.omega)
        .with_height_profile(ua.height)
        .with_cloud_fraction_profile(ua.cloud_fraction)
        // Surface data
        .with_mslp(sd.mslp)
        .with_sfc_temperature(sd.temperature)
        .with_sfc_dew_point(sd.dewpoint)
        .with_station_pressure(sd.station_pres)
        .with_low_cloud(sd.low_cloud)
        .with_mid_cloud(sd.mid_cloud)
        .with_high_cloud(sd.hi_cloud)
        .with_sfc_wind(sd.wind);

    macro_rules! check_and_add {
        ($opt:expr, $key:expr, $hash_map:ident) => {
            if let Some(val) = $opt.into_option() {
                $hash_map.insert($key, val.unpack());
            }
        };
    }

    let mut bufkit_anal: HashMap<&'static str, f64> = HashMap::new();
    check_and_add!(ua.show, "Showalter", bufkit_anal);
    check_and_add!(ua.swet, "SWeT", bufkit_anal);
    check_and_add!(ua.kinx, "K", bufkit_anal);
    check_and_add!(ua.li, "LI", bufkit_anal);
    check_and_add!(ua.lclp, "LCL", bufkit_anal);
    check_and_add!(ua.pwat, "PWAT", bufkit_anal);
    check_and_add!(ua.totl, "TotalTotals", bufkit_anal);
    check_and_add!(ua.cape, "CAPE", bufkit_anal);
    check_and_add!(ua.cins, "CIN", bufkit_anal);
    check_and_add!(ua.lclt, "LCLTemperature", bufkit_anal);
    check_and_add!(ua.eqlv, "EquilibriumLevel", bufkit_anal);
    check_and_add!(ua.lfc, "LFC", bufkit_anal);
    check_and_add!(ua.brch, "BulkRichardsonNumber", bufkit_anal);

    // Add some surface data
    check_and_add!(sd.skin_temp, "SkinTemperature", bufkit_anal);
    check_and_add!(sd.lyr_1_soil_temp, "Layer1SoilTemp", bufkit_anal);
    check_and_add!(sd.snow_1hr, "SnowFall1HourKgPerMeterSquared", bufkit_anal);
    check_and_add!(sd.p01, "Precipitation1HrMm", bufkit_anal);
    check_and_add!(sd.c01, "ConvectivePrecip1HrMm", bufkit_anal);
    check_and_add!(sd.lyr_2_soil_temp, "Layer2SoilTemp", bufkit_anal);
    check_and_add!(sd.snow_ratio, "SnowRatio", bufkit_anal);
    if let Some(WindUV {
        u: MetersPSec(u),
        v: MetersPSec(v),
    }) = sd.storm_motion.into_option()
    {
        bufkit_anal.insert("StormMotionUMps", u);
        bufkit_anal.insert("StormMotionVMps", v);
    }
    check_and_add!(sd.srh, "StormRelativeHelicity", bufkit_anal);

    Analysis::new(snd).with_provider_analysis(bufkit_anal)
}

/// Iterator type for `BufkitData` that returns a `Sounding`.
pub struct SoundingIterator<'a> {
    upper_air_it: UpperAirIterator<'a>,
    surface_it: SurfaceIterator<'a>,
}

impl<'a> Iterator for SoundingIterator<'a> {
    type Item = Analysis;

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
                return Some(combine_data(next_ua, next_sd));
            }
        }
    }
}
