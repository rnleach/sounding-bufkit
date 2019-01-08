//! Parses the *variables* vs height/pressure, or the core part of the sounding.

use crate::error::*;
use crate::parse_util::check_missing;
use metfor::{Celsius, HectoPascal, Kelvin, Knots, Meters, PaPS, WindSpdDir};
use optional::Optioned;
use std::error::Error;

#[derive(Debug)]
pub struct Profile {
    pub pressure: Vec<Optioned<HectoPascal>>,   // Pressure (hPa)
    pub temperature: Vec<Optioned<Celsius>>,    // Temperature (C)
    pub wet_bulb: Vec<Optioned<Celsius>>,       // Wet Bulb (C)
    pub dew_point: Vec<Optioned<Celsius>>,      // Dew Point (C)
    pub theta_e: Vec<Optioned<Kelvin>>,         // Equivalent Potential Temperature (K)
    pub wind: Vec<Optioned<WindSpdDir<Knots>>>, // Wind speed and direction in knots
    pub omega: Vec<Optioned<PaPS>>,             // Pressure vertical velocity (Pa/sec)
    pub height: Vec<Optioned<Meters>>,          // height above MSL in meters
    pub cloud_fraction: Vec<Optioned<f64>>,     // Cloud fraction
}

impl Profile {
    /// Given a String or slice of characters, parse them into an Profile struct.
    pub fn parse(src: &str) -> Result<Profile, Box<dyn Error>> {
        let (header, values) = Profile::split_header_and_values(src)?;
        let cols = Profile::get_column_indexes(header)?;
        Profile::parse_values(values, &cols)
    }

    /// Split the section into the header and values.
    fn split_header_and_values(src: &str) -> Result<(&str, &str), BufkitFileError> {
        // Find the end of the header, and split into header and values.
        let header_end = src
            .find(|c| c == '-' || char::is_digit(c, 10))
            .ok_or_else(BufkitFileError::new)?;
        Ok(src.split_at(header_end))
    }

    /// Get the index of each column name, if it exists
    fn get_column_indexes(header: &str) -> Result<ProfileColIndexes, BufkitFileError> {
        let cols_text = header.trim().split_whitespace();

        let mut cols: ProfileColIndexes = Default::default();

        for (i, val) in cols_text.enumerate() {
            match val.trim() {
                "PRES" => cols.names[i] = ColName::PRES,
                "TMPC" => cols.names[i] = ColName::TMPC,
                "TMWC" => cols.names[i] = ColName::TMWC,
                "DWPC" => cols.names[i] = ColName::DWPC,
                "THTE" => cols.names[i] = ColName::THTE,
                "DRCT" => cols.names[i] = ColName::DRCT,
                "SKNT" => cols.names[i] = ColName::SKNT,
                "OMEG" => cols.names[i] = ColName::OMEG,
                "CFRL" => cols.names[i] = ColName::CFRL,
                "HGHT" => cols.names[i] = ColName::HGHT,
                _ => return Err(BufkitFileError::new()),
            }
        }

        Ok(cols)
    }

    /// Given a string slice of values and some column indexes, parse them!
    fn parse_values(values: &str, cols: &ProfileColIndexes) -> Result<Profile, Box<dyn Error>> {
        use std::str::FromStr;

        // Current GFS soundings have 64 levels of upper air data (2017)
        const INITIAL_CAPACITY: usize = 64;

        let mut parsed_vals = Profile {
            pressure: Vec::with_capacity(INITIAL_CAPACITY),
            temperature: Vec::with_capacity(INITIAL_CAPACITY),
            wet_bulb: Vec::with_capacity(INITIAL_CAPACITY),
            dew_point: Vec::with_capacity(INITIAL_CAPACITY),
            theta_e: Vec::with_capacity(INITIAL_CAPACITY),
            wind: Vec::with_capacity(INITIAL_CAPACITY),
            omega: Vec::with_capacity(INITIAL_CAPACITY),
            height: Vec::with_capacity(INITIAL_CAPACITY),
            cloud_fraction: Vec::with_capacity(INITIAL_CAPACITY),
        };

        let mut direction: Vec<Optioned<f64>> = Vec::with_capacity(INITIAL_CAPACITY);
        let mut speed: Vec<Optioned<Knots>> = Vec::with_capacity(INITIAL_CAPACITY);

        let num_cols = cols.num_cols();
        let values = values.trim().split_whitespace();

        for (i, text_val) in values.enumerate() {
            use self::ColName::*;

            let val = check_missing(f64::from_str(text_val)?);

            match cols.names[i % num_cols] {
                NONE => return Err(BufkitFileError::new().into()),
                PRES => parsed_vals.pressure.push(val.map_t(HectoPascal)),
                TMPC => parsed_vals.temperature.push(val.map_t(Celsius)),
                TMWC => parsed_vals.wet_bulb.push(val.map_t(Celsius)),
                DWPC => parsed_vals.dew_point.push(val.map_t(Celsius)),
                THTE => parsed_vals.theta_e.push(val.map_t(Kelvin)),
                DRCT => direction.push(val),
                SKNT => speed.push(val.map_t(Knots)),
                OMEG => parsed_vals.omega.push(val.map_t(PaPS)),
                CFRL => parsed_vals.cloud_fraction.push(val),
                HGHT => parsed_vals.height.push(val.map_t(Meters)),
            }
        }

        parsed_vals.wind = direction
            .into_iter()
            .zip(speed.into_iter())
            .map(|(dir_opt, spd_opt)| {
                dir_opt.and_then(|dir| {
                    spd_opt.map_t(|spd| WindSpdDir {
                        speed: spd,
                        direction: dir,
                    })
                })
            })
            .collect();

        Ok(parsed_vals)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColName {
    NONE,
    PRES,
    TMPC,
    TMWC,
    DWPC,
    THTE,
    DRCT,
    SKNT,
    OMEG,
    CFRL,
    HGHT,
}

impl Default for ColName {
    fn default() -> ColName {
        ColName::NONE
    }
}

#[derive(Debug, Default)]
pub struct ProfileColIndexes {
    names: [ColName; 10],
}

impl ProfileColIndexes {
    /// Get the number of non-None columns.
    pub fn num_cols(&self) -> usize {
        let mut ncols = 0;

        for &col in &self.names {
            if col != ColName::NONE {
                ncols += 1;
            }
        }

        ncols
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use optional::some;

    #[test]
    fn test_parse() {
        let test_data = "PRES TMPC TMWC DWPC THTE DRCT SKNT OMEG HGHT
                     906.70 10.54 6.12 1.52 305.69 270.00 2.14 -2.00 994.01
                     901.50 10.04 5.79 1.32 305.54 274.76 2.33 -2.00 1041.87";

        let upper_air = Profile::parse(test_data).unwrap();

        println!("upper_air: {:?}", upper_air);

        assert_eq!(
            upper_air.pressure,
            vec![some(HectoPascal(906.7)), some(HectoPascal(901.5))]
        );
        assert_eq!(
            upper_air.temperature,
            vec![some(Celsius(10.54)), some(Celsius(10.04))]
        );
        assert_eq!(
            upper_air.wet_bulb,
            vec![some(Celsius(6.12)), some(Celsius(5.79))]
        );
        assert_eq!(
            upper_air.dew_point,
            vec![some(Celsius(1.52)), some(Celsius(1.32))]
        );
        assert_eq!(
            upper_air.theta_e,
            vec![some(Kelvin(305.69)), some(Kelvin(305.54))]
        );
        assert_eq!(
            upper_air.wind,
            vec![
                some(WindSpdDir {
                    direction: 270.0,
                    speed: Knots(2.14)
                }),
                some(WindSpdDir {
                    direction: 274.76,
                    speed: Knots(2.33)
                })
            ]
        );
        assert_eq!(upper_air.omega, vec![some(PaPS(-2.00)), some(PaPS(-2.00))]);
        assert_eq!(
            upper_air.height,
            vec![some(Meters(994.01)), some(Meters(1041.87))]
        );
    }

    // PRES - Pressure (hPa)
    // TMPC - Temperature (C)
    // TMWC - Wet bulb temperature (C)
    // DWPC - Dewpoint (C)
    // THTE - Equivalent potential temperature (K)
    // DRCT - Wind direction (degrees)
    // SKNT - Wind speed (knots)
    // OMEG - Vertical velocity (Pa/s)
    // CFRL - Fractional cloud coverage (percent)
    // HGHT - Height of pressure level (m)

    #[test]
    fn test_split_header_and_values() {
        let test_data = "PRES TMPC TMWC DWPC THTE DRCT SKNT OMEG HGHT
                     906.70 10.54 6.12 1.52 305.69 270.00 2.14 -2.00 994.01 \
                     901.50 10.04 5.79 1.32 305.54 274.76 2.33 -2.00 1041.87";

        let (head, values) = Profile::split_header_and_values(test_data).unwrap();

        println!("head: {}", head);
        assert_eq!(head.trim(), "PRES TMPC TMWC DWPC THTE DRCT SKNT OMEG HGHT");
        assert_eq!(
            values,
            "906.70 10.54 6.12 1.52 305.69 270.00 2.14 -2.00 994.01 \
             901.50 10.04 5.79 1.32 305.54 274.76 2.33 -2.00 1041.87"
        );
    }

    #[test]
    fn test_get_column_indexes() {
        use self::ColName::*;

        let test_data = "PRES TMPC TMWC DWPC THTE DRCT SKNT OMEG HGHT ";

        let cols = Profile::get_column_indexes(test_data).unwrap();

        println!("cols: {:?}", cols);
        assert_eq!(cols.names[0], PRES);
        assert_eq!(cols.names[1], TMPC);
        assert_eq!(cols.names[2], TMWC);
        assert_eq!(cols.names[3], DWPC);
        assert_eq!(cols.names[4], THTE);
        assert_eq!(cols.names[5], DRCT);
        assert_eq!(cols.names[6], SKNT);
        assert_eq!(cols.names[7], OMEG);
        assert_eq!(cols.names[8], HGHT);
        assert_eq!(cols.names[9], NONE);
    }

    #[test]
    fn test_parse_values() {
        use self::ColName::*;

        let test_data = "906.70 10.54 6.12 1.52 305.69 270.00 2.14 -2.00 994.01
                     901.50 10.04 5.79 1.32 305.54 274.76 2.33 -2.00 1041.87";

        let cols = ProfileColIndexes {
            names: [PRES, TMPC, TMWC, DWPC, THTE, DRCT, SKNT, OMEG, HGHT, NONE],
        };

        let upper_air = Profile::parse_values(test_data, &cols).unwrap();

        println!("upper_air: {:?}", upper_air);

        assert_eq!(
            upper_air.pressure,
            vec![some(HectoPascal(906.7)), some(HectoPascal(901.5))]
        );
        assert_eq!(
            upper_air.temperature,
            vec![some(Celsius(10.54)), some(Celsius(10.04))]
        );
        assert_eq!(
            upper_air.wet_bulb,
            vec![some(Celsius(6.12)), some(Celsius(5.79))]
        );
        assert_eq!(
            upper_air.dew_point,
            vec![some(Celsius(1.52)), some(Celsius(1.32))]
        );
        assert_eq!(
            upper_air.theta_e,
            vec![some(Kelvin(305.69)), some(Kelvin(305.54))]
        );
        assert_eq!(
            upper_air.wind,
            vec![
                some(WindSpdDir {
                    direction: 270.0,
                    speed: Knots(2.14)
                }),
                some(WindSpdDir {
                    direction: 274.76,
                    speed: Knots(2.33)
                })
            ]
        );
        assert_eq!(upper_air.omega, vec![some(PaPS(-2.00)), some(PaPS(-2.00))]);
        assert_eq!(
            upper_air.height,
            vec![some(Meters(994.01)), some(Meters(1041.87))]
        );
    }
}
