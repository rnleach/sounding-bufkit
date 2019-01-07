//! Module for parsing surface data in a bufkit file.

use crate::error::*;
use chrono::{NaiveDate, NaiveDateTime};
use std::error::Error;

/// Surface data.
#[derive(Debug, PartialEq)]
pub struct SurfaceData {
    pub station_num: i32,          // Same is in StationInfo
    pub valid_time: NaiveDateTime, // Always assume UTC.
    pub mslp: f64,                 // Surface pressure reduce to mean sea level
    pub station_pres: f64,         // Surface pressure
    pub low_cloud: f64,            // low cloud coverage percent
    pub mid_cloud: f64,            // mid cloud coverage percent
    pub hi_cloud: f64,             // high cloud coverage percent
    pub uwind: f64,                // zonal surface wind (m/s)
    pub vwind: f64,                // meridional surface wind (m/s)
    pub temperature: f64,          // 2 meter temperature C
    pub dewpoint: f64,             // 2 meter dew point C

    pub skin_temp: f64,       // Skin temperature (C)
    pub lyr_1_soil_temp: f64, // Layer 1 soil temperature (K)
    pub snow_1hr: f64,        // 1-hour accumulated snowfall (Kg/m**2)

    // WTNS - Soil moisture availability (percent)
    pub p01: f64,             // P01M - 1-hour total precipitation (mm)
    pub c01: f64,             // C01M - 1-hour convective precipitation (mm)
    pub lyr_2_soil_temp: f64, // STC2 - Layer 2 soil temperature (K)
    pub snow_ratio: f64,      // SNRA - Snow ratio from explicit cloud scheme (percent)
    // R01M - 1-hour accumulated surface runoff (mm)
    // BFGR - 1-hour accumulated baseflow-groundwater runoff (mm)
    // Q2MS - 2-meter specific humidity
    // WXTS - Snow precipitation type (1=Snow)
    // WXTP - Ice pellets precipitation type (1=Ice pellets)
    // WXTZ - Freezing rain precipitation type (1=Freezing rain)
    // WXTR - Rain precipitation type (1=Rain)
    pub u_storm: f64, // USTM - U-component of storm motion (m/s)
    pub v_storm: f64, // VSTM - V-component of storm motion (m/s)
    pub srh: f64,     // HLCY - Storm relative helicity (m**2/s**2)
                      // SLLH - 1-hour surface evaporation (mm)
                      // WSYM - Weather type symbol number
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
                "USTM" => cols.names.push(USTM),
                "VSTM" => cols.names.push(VSTM),
                "HLCY" => cols.names.push(HLCY),
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

        for i in 0..cols.num_cols() {
            if let Some(token) = tokens.next() {
                use self::SfcColName::*;
                use crate::parse_util::*;
                let _dummy: f64; // Used just to check that there is a valid value there.

                match cols.names[i] {
                    NONE => _dummy = f64::from_str(token)?,
                    STN => sd.station_num = i32::from_str(token)?,
                    VALIDTIME => sd.valid_time = parse_naive_date_time(token)?,
                    PMSL => sd.mslp = f64::from_str(token)?,
                    PRES => sd.station_pres = f64::from_str(token)?,
                    LCLD => sd.low_cloud = f64::from_str(token)?,
                    MCLD => sd.mid_cloud = f64::from_str(token)?,
                    HCLD => sd.hi_cloud = f64::from_str(token)?,
                    UWND => sd.uwind = f64::from_str(token)?,
                    VWND => sd.vwind = f64::from_str(token)?,
                    T2MS => sd.temperature = f64::from_str(token)?,
                    TD2M => sd.dewpoint = f64::from_str(token)?,
                    SKTC => sd.skin_temp = f64::from_str(token)?,
                    STC1 => sd.lyr_1_soil_temp = f64::from_str(token)?,
                    SNFL => sd.snow_1hr = f64::from_str(token)?,
                    P01M => sd.p01 = f64::from_str(token)?,
                    C01M => sd.c01 = f64::from_str(token)?,
                    STC2 => sd.lyr_2_soil_temp = f64::from_str(token)?,
                    SNRA => sd.snow_ratio = f64::from_str(token)?,
                    USTM => sd.u_storm = f64::from_str(token)?,
                    VSTM => sd.v_storm = f64::from_str(token)?,
                    HLCY => sd.srh = f64::from_str(token)?,
                };
            } else {
                return Err(BufkitFileError::new().into());
            }
        }

        Ok(sd)
    }
}

impl Default for SurfaceData {
    fn default() -> SurfaceData {
        SurfaceData {
            station_num: ::std::i32::MIN,
            valid_time: NaiveDate::from_ymd(0, 1, 1).and_hms(0, 0, 0),
            mslp: -9999.0,
            station_pres: -9999.0,
            low_cloud: -9999.0,
            mid_cloud: -9999.0,
            hi_cloud: -9999.0,
            uwind: -9999.0,
            vwind: -9999.0,
            temperature: -9999.0,
            dewpoint: -9999.0,
            skin_temp: -9999.0,
            lyr_1_soil_temp: -9999.0,
            snow_1hr: -9999.0,
            p01: -9999.0,
            c01: -9999.0,
            lyr_2_soil_temp: -9999.0,
            snow_ratio: -9999.0,
            u_storm: -9999.0,
            v_storm: -9999.0,
            srh: -9999.0,
        }
    }
}

/// There are many more columns that can be parsed, but these are all being parsed for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SfcColName {
    NONE,
    STN,
    VALIDTIME,
    PMSL,
    PRES,
    LCLD,
    MCLD,
    HCLD,
    UWND,
    VWND,
    T2MS, // 2 Meter temperature
    TD2M, // 2 Meter dew point
    SKTC, // Skin temperature
    STC1, // layer 1 soil temperature
    SNFL, // 1-hour snow fall kg/m^2
    P01M, // 1-hour total precipitation (mm)
    C01M, // 1-hour convective precipitation (mm)
    STC2, // Layer 2 soil temperature (K)
    SNRA, // Snow ratio from explicit cloud scheme (percent)
    USTM, // USTM - U-component of storm motion (m/s)
    VSTM, // VSTM - V-component of storm motion (m/s)
    HLCY, // HLCY - Storm relative helicity (m**2/s**2)
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

                25 => col_name = USTM,
                26 => col_name = VSTM,
                27 => col_name = HLCY,

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
