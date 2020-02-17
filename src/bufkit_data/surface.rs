//! Module for parsing surface data in a bufkit file.

use crate::error::*;
use chrono::{NaiveDate, NaiveDateTime};
use metfor::{Celsius, HectoPascal, Kelvin, Knots, MetersPSec, Mm, WindSpdDir, WindUV};
use optional::{none, some, Optioned};
use std::error::Error;

/// Surface data.
#[derive(Debug, PartialEq)]
pub struct SurfaceData {
    pub station_num: i32,                    // Same is in StationInfo
    pub valid_time: NaiveDateTime,           // Always assume UTC.
    pub mslp: Optioned<HectoPascal>,         // Surface pressure reduce to mean sea level
    pub station_pres: Optioned<HectoPascal>, // Surface pressure
    pub low_cloud: Optioned<f64>,            // low cloud coverage percent
    pub mid_cloud: Optioned<f64>,            // mid cloud coverage percent
    pub hi_cloud: Optioned<f64>,             // high cloud coverage percent
    pub wind: Optioned<WindSpdDir<Knots>>,   // surface wind direction and speed
    pub temperature: Optioned<Celsius>,      // 2 meter temperature C
    pub dewpoint: Optioned<Celsius>,         // 2 meter dew point C

    pub skin_temp: Optioned<Celsius>,      // Skin temperature (C)
    pub lyr_1_soil_temp: Optioned<Kelvin>, // Layer 1 soil temperature (K)
    pub snow_1hr: Optioned<f64>,           // 1-hour accumulated snowfall (Kg/m**2)

    // WTNS - Soil moisture availability (percent)
    pub p01: Optioned<Mm>, // P01M - 1-hour total precipitation (mm)
    pub c01: Optioned<Mm>, // C01M - 1-hour convective precipitation (mm)
    pub lyr_2_soil_temp: Optioned<Kelvin>, // STC2 - Layer 2 soil temperature (K)
    pub snow_ratio: Optioned<f64>, // SNRA - Snow ratio from explicit cloud scheme (percent)
    // R01M - 1-hour accumulated surface runoff (mm)
    // BFGR - 1-hour accumulated baseflow-groundwater runoff (mm)
    // Q2MS - 2-meter specific humidity
    pub snow_type: Option<bool>, // WXTS - Snow precipitation type (1=Snow)
    pub ice_pellets_type: Option<bool>, // WXTP - Ice pellets precipitation type (1=Ice pellets)
    pub fzra_type: Option<bool>, // WXTZ - Freezing rain precipitation type (1=Freezing rain)
    pub rain_type: Option<bool>, // WXTR - Rain precipitation type (1=Rain)
    pub storm_motion: Optioned<WindUV<MetersPSec>>, // Storm motion (m/s)
    pub srh: Optioned<f64>,      // HLCY - Storm relative helicity (m**2/s**2)
    // SLLH - 1-hour surface evaporation (mm)
    pub wx_sym_cod: Optioned<f64>, // WSYM - Weather type symbol number
                                   // CDBP - Pressure at the base of cloud (hPa)
                                   // VSBK - Visibility (km)
}

impl SurfaceData {
    /// Get the index of each column name, if it exists.
    ///
    /// This function does not match all possible column names. Much more work would need to be
    /// done for that, but there are some relavent links in the bufkit_parameters.txt file.
    pub fn parse_columns(header: &str) -> Result<SfcColumns, BufkitFileError> {
        use self::SfcColName::*;

        let cols_text = header.trim().split_whitespace();

        let mut cols = SfcColumns {
            names: Vec::with_capacity(33),
        };

        for val in cols_text {
            match val.trim() {
                "STN" => cols.names.push(STN),
                "YYMMDD/HHMM" => cols.names.push(VALIDTIME),
                "PMSL" => cols.names.push(PMSL),
                "PRES" => cols.names.push(PRES),
                "LCLD" => cols.names.push(LCLD),
                "MCLD" => cols.names.push(MCLD),
                "HCLD" => cols.names.push(HCLD),
                "UWND" => cols.names.push(UWND),
                "VWND" => cols.names.push(VWND),
                "T2MS" => cols.names.push(T2MS),
                "TD2M" => cols.names.push(TD2M),
                "SKTC" => cols.names.push(SKTC),
                "STC1" => cols.names.push(STC1),
                "SNFL" => cols.names.push(SNFL),
                "P01M" => cols.names.push(P01M),
                "C01M" => cols.names.push(C01M),
                "STC2" => cols.names.push(STC2),
                "SNRA" => cols.names.push(SNRA),
                "WXTS" => cols.names.push(WXTS),
                "WXTP" => cols.names.push(WXTP),
                "WXTZ" => cols.names.push(WXTZ),
                "WXTR" => cols.names.push(WXTR),
                "USTM" => cols.names.push(USTM),
                "VSTM" => cols.names.push(VSTM),
                "HLCY" => cols.names.push(HLCY),
                "WSYM" => cols.names.push(WSYM),
                _ => cols.names.push(NONE),
            }
        }

        // Check that we found some required columns.
        {
            let names: &Vec<_> = &cols.names;
            if names.iter().find(|&&x| x == STN).is_none()
                || names.iter().find(|&&x| x == VALIDTIME).is_none()
            {
                return Err(BufkitFileError::new());
            }
        }

        Ok(cols)
    }

    /// Parse a few values stored as strings in the `tokens` iterator.
    pub fn parse_values(tokens: &str, cols: &SfcColumns) -> Result<SurfaceData, Box<dyn Error>> {
        use std::str::FromStr;
        let mut tokens = tokens.split_whitespace();

        let mut sd = SurfaceData::default();

        let mut u_wind: Optioned<MetersPSec> = none();
        let mut v_wind: Optioned<MetersPSec> = none();

        let mut u_storm: Optioned<MetersPSec> = none();
        let mut v_storm: Optioned<MetersPSec> = none();

        for i in 0..cols.num_cols() {
            if let Some(token) = tokens.next() {
                use self::SfcColName::*;
                use crate::parse_util::*;
                let _dummy: f64; // Used just to check that there is a valid value there.

                match cols.names[i] {
                    NONE => _dummy = f64::from_str(token)?,
                    STN => sd.station_num = i32::from_str(token)?,
                    VALIDTIME => sd.valid_time = parse_naive_date_time(token)?,
                    PMSL => sd.mslp = check_missing(f64::from_str(token)?).map_t(HectoPascal),
                    PRES => {
                        sd.station_pres = check_missing(f64::from_str(token)?).map_t(HectoPascal)
                    }
                    LCLD => {
                        sd.low_cloud = check_missing(f64::from_str(token)?).map_t(|val| val / 100.0)
                    }
                    MCLD => {
                        sd.mid_cloud = check_missing(f64::from_str(token)?).map_t(|val| val / 100.0)
                    }
                    HCLD => {
                        sd.hi_cloud = check_missing(f64::from_str(token)?).map_t(|val| val / 100.0)
                    }
                    UWND => u_wind = check_missing(f64::from_str(token)?).map_t(MetersPSec),
                    VWND => v_wind = check_missing(f64::from_str(token)?).map_t(MetersPSec),
                    T2MS => sd.temperature = check_missing(f64::from_str(token)?).map_t(Celsius),
                    TD2M => sd.dewpoint = check_missing(f64::from_str(token)?).map_t(Celsius),
                    SKTC => sd.skin_temp = check_missing(f64::from_str(token)?).map_t(Celsius),
                    STC1 => sd.lyr_1_soil_temp = check_missing(f64::from_str(token)?).map_t(Kelvin),
                    SNFL => sd.snow_1hr = check_missing(f64::from_str(token)?),
                    P01M => sd.p01 = check_missing(f64::from_str(token)?).map_t(Mm),
                    C01M => sd.c01 = check_missing(f64::from_str(token)?).map_t(Mm),
                    STC2 => sd.lyr_2_soil_temp = check_missing(f64::from_str(token)?).map_t(Kelvin),
                    SNRA => sd.snow_ratio = check_missing(f64::from_str(token)?),
                    WXTS => {
                        sd.snow_type = check_missing(f64::from_str(token)?).map(|val| val > 0.5)
                    }
                    WXTP => {
                        sd.ice_pellets_type =
                            check_missing(f64::from_str(token)?).map(|val| val > 0.5)
                    }
                    WXTZ => {
                        sd.fzra_type = check_missing(f64::from_str(token)?).map(|val| val > 0.5)
                    }
                    WXTR => {
                        sd.rain_type = check_missing(f64::from_str(token)?).map(|val| val > 0.5)
                    }
                    USTM => u_storm = check_missing(f64::from_str(token)?).map_t(MetersPSec),
                    VSTM => v_storm = check_missing(f64::from_str(token)?).map_t(MetersPSec),
                    HLCY => sd.srh = check_missing(f64::from_str(token)?),
                    WSYM => {
                        sd.wx_sym_cod = if let Ok(val) = f64::from_str(token) {
                            if val == MISSING_F64_INDEX || val == MISSING_F64 {
                                none()
                            } else {
                                some(val)
                            }
                        } else {
                            none()
                        }
                    }
                };
            } else {
                return Err(BufkitFileError::new().into());
            }
        }

        sd.wind = u_wind.and_then(|u| v_wind.map_t(|v| WindSpdDir::<Knots>::from(WindUV { u, v })));
        sd.storm_motion = u_storm.and_then(|u| v_storm.map_t(|v| WindUV { u, v }));

        Ok(sd)
    }
}

impl Default for SurfaceData {
    fn default() -> SurfaceData {
        SurfaceData {
            station_num: ::std::i32::MIN,
            valid_time: NaiveDate::from_ymd(0, 1, 1).and_hms(0, 0, 0),
            mslp: none(),
            station_pres: none(),
            low_cloud: none(),
            mid_cloud: none(),
            hi_cloud: none(),
            wind: none(),
            temperature: none(),
            dewpoint: none(),
            skin_temp: none(),
            lyr_1_soil_temp: none(),
            snow_1hr: none(),
            p01: none(),
            c01: none(),
            lyr_2_soil_temp: none(),
            snow_ratio: none(),
            ice_pellets_type: None,
            snow_type: None,
            fzra_type: None,
            rain_type: None,
            storm_motion: none(),
            srh: none(),
            wx_sym_cod: none(),
        }
    }
}

/// There are many more columns that can be parsed, but these are all being parsed for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SfcColName {
    NONE,
    STN,
    VALIDTIME,
    PMSL, // Mean sea level pressure
    PRES, // Station pressure
    LCLD, // Low cloud amount
    MCLD, // Mid-level cloud amount
    HCLD, // High cloud amount
    UWND, // U-component of the wind
    VWND, // V-component of the wind
    T2MS, // 2 Meter temperature
    TD2M, // 2 Meter dew point
    SKTC, // Skin temperature
    STC1, // layer 1 soil temperature
    SNFL, // 1-hour snow fall kg/m^2
    P01M, // 1-hour total precipitation (mm)
    C01M, // 1-hour convective precipitation (mm)
    STC2, // Layer 2 soil temperature (K)
    SNRA, // Snow ratio from explicit cloud scheme (percent)
    WXTS, // Snow weather type
    WXTP, // Ice pellets weather type
    WXTZ, // Freezing rain weather type,
    WXTR, // Rain weather type,
    USTM, // USTM - U-component of storm motion (m/s)
    VSTM, // VSTM - V-component of storm motion (m/s)
    HLCY, // HLCY - Storm relative helicity (m**2/s**2)
    WSYM, // WSYM - Weather type symbol number
}

#[derive(Debug)]
pub struct SfcColumns {
    names: Vec<SfcColName>,
}

impl SfcColumns {
    /// Get the number of columns.
    pub fn num_cols(&self) -> usize {
        self.names.len()
    }
}

// STN  - 6-digit station number
// YYMMDD/HHMM - Valid time (UTC) in numeric format
// PMSL - Mean sea level pressure (hPa)
// PRES - Station pressure (hPa)
// SKTC - Skin temperature (C)
// STC1 - Layer 1 soil temperature (K)
// SNFL - 1-hour accumulated snowfall (Kg/m**2)
// WTNS - Soil moisture availability (percent)
// P01M - 1-hour total precipitation (mm)
// C01M - 1-hour convective precipitation (mm)
// STC2 - Layer 2 soil temperature (K)
// LCLD - Low cloud coverage (percent)
// MCLD - Middle cloud coverage (percent)
// HCLD - High cloud coverage (percent)
// SNRA - Snow ratio from explicit cloud scheme (percent)
// UWND - 10-meter U wind component (m/s)
// VWND - 10-meter V wind component (m/s)
// R01M - 1-hour accumulated surface runoff (mm)
// BFGR - 1-hour accumulated baseflow-groundwater runoff (mm)
// T2MS - 2-meter temperature (C)
// Q2MS - 2-meter specific humidity
// WXTS - Snow precipitation type (1=Snow)
// WXTP - Ice pellets precipitation type (1=Ice pellets)
// WXTZ - Freezing rain precipitation type (1=Freezing rain)
// WXTR - Rain precipitation type (1=Rain)
// USTM - U-component of storm motion (m/s)
// VSTM - V-component of storm motion (m/s)
// HLCY - Storm relative helicity (m**2/s**2)
// SLLH - 1-hour surface evaporation (mm)
// EVAP - Evaportaion, units not given, mm?
// WSYM - Weather type symbol number
// CDBP - Pressure at the base of cloud (hPa)
// VSBK - Visibility (km)
// TD2M - 2-meter dewpoint (C)
// more paramters than listed here!

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_columns() {
        use self::SfcColName::*;

        let test_data = "STN YYMMDD/HHMM PMSL PRES SKTC STC1 EVAP P03M C03M SWEM LCLD MCLD HCLD \
                         UWND VWND T2MS Q2MS WXTS WXTP WXTZ WXTR S03M TD2M ";

        let col_idx = SurfaceData::parse_columns(test_data).unwrap();

        assert_eq!(col_idx.num_cols(), 23);

        for i in 1..col_idx.names.len() {
            let col_name: SfcColName;
            match i {
                0 => col_name = STN,
                1 => col_name = VALIDTIME,
                2 => col_name = PMSL,
                3 => col_name = PRES,
                4 => col_name = SKTC,
                5 => col_name = STC1,
                10 => col_name = LCLD,
                11 => col_name = MCLD,
                12 => col_name = HCLD,
                13 => col_name = UWND,
                14 => col_name = VWND,
                15 => col_name = T2MS,
                17 => col_name = WXTS,
                18 => col_name = WXTP,
                19 => col_name = WXTZ,
                20 => col_name = WXTR,
                22 => col_name = TD2M,
                _ => col_name = NONE,
            };

            assert_eq!(col_idx.names[i], col_name);
        }

        let test_data = "STN YYMMDD/HHMM PMSL PRES SKTC STC1 SNFL WTNS P01M C01M STC2 LCLD MCLD \
                         HCLD SNRA UWND VWND R01M BFGR T2MS Q2MS WXTS WXTP WXTZ WXTR USTM VSTM \
                         HLCY SLLH WSYM CDBP VSBK TD2M ";

        let col_idx = SurfaceData::parse_columns(test_data).unwrap();

        assert_eq!(col_idx.num_cols(), 33);

        for i in 1..col_idx.names.len() {
            let col_name: SfcColName;
            match i {
                0 => col_name = STN,
                1 => col_name = VALIDTIME,
                2 => col_name = PMSL,
                3 => col_name = PRES,
                4 => col_name = SKTC,
                5 => col_name = STC1,
                6 => col_name = SNFL,

                8 => col_name = P01M,
                9 => col_name = C01M,
                10 => col_name = STC2,
                11 => col_name = LCLD,
                12 => col_name = MCLD,
                13 => col_name = HCLD,
                14 => col_name = SNRA,
                15 => col_name = UWND,
                16 => col_name = VWND,

                19 => col_name = T2MS,

                21 => col_name = WXTS,
                22 => col_name = WXTP,
                23 => col_name = WXTZ,
                24 => col_name = WXTR,
                25 => col_name = USTM,
                26 => col_name = VSTM,
                27 => col_name = HLCY,

                29 => col_name = WSYM,

                32 => col_name = TD2M,

                _ => col_name = NONE,
            };

            assert_eq!(col_idx.names[i], col_name);
        }

        // Test for error condition on missing valid time.
        let test_data = "STN PMSL PRES SKTC STC1 SNFL WTNS P01M C01M STC2 LCLD MCLD HCLD SNRA \
                         UWND VWND R01M BFGR T2MS Q2MS WXTS WXTP WXTZ WXTR USTM VSTM HLCY SLLH \
                         WSYM CDBP VSBK TD2M ";

        assert!(SurfaceData::parse_columns(test_data).is_err());
    }
}
