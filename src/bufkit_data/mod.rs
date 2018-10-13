//! Module for reading a bufkit file and breaking it into smaller pieces for parsing later.
use std::collections::HashMap;
use std::path::Path;

use optional::{none, some, Optioned};

mod surface;
mod surface_section;
mod upper_air;
mod upper_air_section;

use sounding_analysis::Analysis;
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
    pub fn validate_file_format(&self) -> Result<(), Error> {
        let data = self.data()?;
        data.validate()?;

        Ok(())
    }

    /// Get a bufkit data object from this file.
    pub fn data(&self) -> Result<BufkitData, Error> {
        BufkitData::new(&self.file_text)
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
    type Item = Analysis;
    type IntoIter = SoundingIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SoundingIterator {
            upper_air_it: self.upper_air.into_iter(),
            surface_it: self.surface.into_iter(),
        }
    }
}

fn combine_data(ua: &UpperAir, sd: &SurfaceData) -> Analysis {
    use sounding_base::Profile;
    use sounding_base::Surface;

    // Missing or no data values used in Bufkit files
    const MISSING_I32: i32 = -9999;
    const MISSING_F64: f64 = -9999.0;

    fn check_missing(val: f64) -> Optioned<f64> {
        if val == MISSING_F64 {
            none()
        } else {
            some(val)
        }
    }

    fn check_missing_i32(val: i32) -> Option<i32> {
        if val == MISSING_I32 {
            None
        } else {
            Some(val)
        }
    }

    let coords: Option<(f64, f64)> = if ua.lat == MISSING_F64 || ua.lon == MISSING_F64 {
        None
    } else {
        Some((ua.lat, ua.lon))
    };

    let station = StationInfo::new_with_values(
        check_missing_i32(ua.num),
        coords,
        check_missing(ua.elevation),
    );

    let (sfc_wind_dir, sfc_wind_spd) = {
        let u = check_missing(sd.uwind).into_option();
        let v = check_missing(sd.vwind).into_option();
        if let (Some(u), Some(v)) = (u,v){
            let (dir, spd) = ::metfor::uv_to_spd_dir(u, v);
            (some(dir), some(spd))
        } else {
            (none(), none())
        }
    };

    let strm_motion_spd = check_missing(sd.u_storm)
        .and_then(|u| check_missing(sd.v_storm).and_then(|v| some(u.hypot(v))))
        .and_then(|mps| some(mps * 1.94384)); // convert m/s to knots

    let strm_motion_dir = check_missing(sd.u_storm)
        .and_then(|u| check_missing(sd.v_storm).and_then(|v| some(v.atan2(u).to_degrees())))
        .and_then(|mut dir| {
            // map into 0 -> 360 range.
            while dir < 0.0 {
                dir += 360.0;
            }
            while dir > 360.0 {
                dir -= 360.0;
            }
            some(dir)
        });

    let snd = Sounding::new()
        .set_station_info(station)
        .set_valid_time(ua.valid_time)
        .set_lead_time(check_missing_i32(ua.lead_time))

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
        .set_surface_value(Surface::Temperature, check_missing(sd.temperature))
        .set_surface_value(Surface::DewPoint, check_missing(sd.dewpoint))
        .set_surface_value(Surface::StationPressure, check_missing(sd.station_pres))
        .set_surface_value(Surface::LowCloud, check_missing(sd.low_cloud))
        .set_surface_value(Surface::MidCloud, check_missing(sd.mid_cloud))
        .set_surface_value(Surface::HighCloud, check_missing(sd.hi_cloud))
        .set_surface_value(Surface::WindDirection, sfc_wind_dir)
        .set_surface_value(Surface::WindSpeed, sfc_wind_spd);

    macro_rules! check_and_add {
        ($opt:expr, $key:expr, $hash_map:ident) => {
            if let Some(val) = check_missing($opt).into() {
                $hash_map.insert($key, val);
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
    check_and_add!(sd.p01 / 25.4, "Precipitation1HrIn", bufkit_anal);
    check_and_add!(sd.c01 / 25.4, "ConvectivePrecip1HrIn", bufkit_anal);
    check_and_add!(sd.lyr_2_soil_temp, "Layer2SoilTemp", bufkit_anal);
    check_and_add!(sd.snow_ratio, "SnowRatio", bufkit_anal);
    if let (Some(spd), Some(dir)) = (strm_motion_spd.into(), strm_motion_dir.into()) {
        bufkit_anal.insert("StormMotionSpd", spd);
        bufkit_anal.insert("StormMotionDir", dir);
    }
    check_and_add!(sd.srh, "StormRelativeHelicity", bufkit_anal);

    let anal = Analysis::new(snd).with_provider_analysis(bufkit_anal);

    anal
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
                return Some(combine_data(&next_ua, &next_sd));
            }
        }
    }
}
