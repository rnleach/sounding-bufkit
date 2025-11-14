//! Module for parsing the upper air section of a bufkit file.

mod indexes;
mod profile;
mod station_info;

use crate::error::*;
use chrono::NaiveDateTime;
use metfor::{
    Celsius, CelsiusDiff, HectoPascal, JpKg, Kelvin, Knots, Meters, Mm, PaPS, WindSpdDir,
};
use optional::Optioned;
use std::error::Error;

/// All the values from a parsed sounding in one struct.
#[derive(Debug)]
pub struct UpperAir {
    // Station info
    pub num: i32,                    // station number, USAF number, eg 727730
    pub valid_time: NaiveDateTime,   // valid time of sounding
    pub lead_time: i32,              // Forecast lead time in hours from model init
    pub id: Option<String>,          // Usually a 3 or 4 letter alpha numeric designation.
    pub lat: Optioned<f64>,          // latitude
    pub lon: Optioned<f64>,          // longitude
    pub elevation: Optioned<Meters>, // elevation (m)

    // Indexes
    pub show: Optioned<CelsiusDiff>, // Showalter index
    pub li: Optioned<CelsiusDiff>,   // Lifted index
    pub swet: Optioned<f64>,         // Severe Weather Threat index
    pub kinx: Optioned<Celsius>,     // K-index
    pub lclp: Optioned<HectoPascal>, // Lifting Condensation Level (hPa)
    pub pwat: Optioned<Mm>,          // Precipitable water (mm)
    pub totl: Optioned<f64>,         // Total-Totals
    pub cape: Optioned<JpKg>,        // Convective Available Potential Energy
    pub lclt: Optioned<Kelvin>,      // Temperature at LCL (K)
    pub cins: Optioned<JpKg>,        // Convective Inhibitive Energy
    pub eqlv: Optioned<HectoPascal>, // Equilibrium Level (hPa)
    pub lfc: Optioned<HectoPascal>,  // Level of Free Convection (hPa)
    pub brch: Optioned<f64>,         // Bulk Richardson Number

    // Upper air
    pub pressure: Vec<Optioned<HectoPascal>>, // Pressure (hPa)
    pub temperature: Vec<Optioned<Celsius>>,  // Temperature (C)
    pub wet_bulb: Vec<Optioned<Celsius>>,     // Wet Bulb (C)
    pub dew_point: Vec<Optioned<Celsius>>,    // Dew Point (C)
    pub theta_e: Vec<Optioned<Kelvin>>,       // Equivalent Potential Temperature (K)
    pub wind: Vec<Optioned<WindSpdDir<Knots>>>, // Wind speed and direction, knots
    pub omega: Vec<Optioned<PaPS>>,           // Pressure vertical velocity (Pa/sec)
    pub height: Vec<Optioned<Meters>>,        // height above MSL in meters
    pub cloud_fraction: Vec<Optioned<f64>>,   // Cloud fraction
}

impl UpperAir {
    /// Given a string slice, attempt to parse it into a UpperAir.
    pub fn parse(text: &str) -> Result<UpperAir, Box<dyn Error>> {
        use self::indexes::Indexes;
        use self::profile::Profile;
        use self::station_info::StationInfo;
        use crate::parse_util::find_blank_line;

        let mut break_point = find_blank_line(text).ok_or_else(BufkitFileError::new)?;
        let (station_info_section, the_rest) = text.split_at(break_point);

        break_point = find_blank_line(the_rest).ok_or_else(BufkitFileError::new)?;
        let (index_section, upper_air_section) = the_rest.split_at(break_point);

        let station_info = StationInfo::parse(station_info_section)?;
        let indexes = Indexes::parse(index_section)?;
        let upper_air = Profile::parse(upper_air_section)?;

        Ok(UpperAir {
            // Station info
            num: station_info.num,
            valid_time: station_info.valid_time,
            lead_time: station_info.lead_time,
            id: station_info.id,
            lat: station_info.lat,
            lon: station_info.lon,
            elevation: station_info.elevation,

            // Indexes
            show: indexes.show,
            li: indexes.li,
            swet: indexes.swet,
            kinx: indexes.kinx,
            lclp: indexes.lclp,
            pwat: indexes.pwat,
            totl: indexes.totl,
            cape: indexes.cape,
            lclt: indexes.lclt,
            cins: indexes.cins,
            eqlv: indexes.eqlv,
            lfc: indexes.lfc,
            brch: indexes.brch,

            // Upper air
            pressure: upper_air.pressure,
            temperature: upper_air.temperature,
            wet_bulb: upper_air.wet_bulb,
            dew_point: upper_air.dew_point,
            theta_e: upper_air.theta_e,
            wind: upper_air.wind,
            omega: upper_air.omega,
            height: upper_air.height,
            cloud_fraction: upper_air.cloud_fraction,
        })
    }

    /// Validate the sounding
    pub fn validate(&self) -> Result<(), BufkitFileError> {
        // Pressure is mandatory
        let len = self.pressure.len();
        if len == 0 {
            return Err(BufkitFileError::new());
        }

        let is_valid_length = |l| {
            if l == 0 || l == len {
                Ok(())
            } else {
                Err(BufkitFileError::new())
            }
        };

        is_valid_length(self.temperature.len())?;
        is_valid_length(self.wet_bulb.len())?;
        is_valid_length(self.dew_point.len())?;
        is_valid_length(self.theta_e.len())?;
        is_valid_length(self.wind.len())?;
        is_valid_length(self.omega.len())?;
        is_valid_length(self.height.len())?;
        is_valid_length(self.cloud_fraction.len())?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_test_data() -> &'static str {
        "STID = KMSO STNM = 727730 TIME = 170401/0100
         SLAT = 46.87 SLON = -114.16 SELV = 1335.0
         STIM = 1

         SHOW = 8.12 LIFT = 8.00 SWET = 39.08 KINX = 14.88
         LCLP = 780.77 PWAT = 9.28 TOTL = 39.55 CAPE = 0.00
         LCLT = 272.88 CINS = 0.00 EQLV = -9999.00 LFCT = -9999.00
         BRCH = 0.00

         PRES TMPC TMWC DWPC THTE DRCT SKNT OMEG
         CFRL HGHT
         867.20 8.04 4.71 1.19 307.17 288.43 2.45 0.00
         0.00 1353.07
         863.50 7.64 4.42 0.99 306.96 293.63 3.40 0.00
         0.00 1388.34
         859.80 7.24 4.18 0.90 306.87 292.38 3.57 0.00
         0.00 1423.71
         856.10 6.94 3.99 0.81 306.89 292.38 3.57 0.00
         0.00 1459.19
         852.20 6.54 3.75 0.72 306.83 293.63 3.40 0.00
         0.00 1496.70
         848.30 6.14 3.52 0.65 306.78 293.63 3.40 0.00
         0.00 1534.33
         844.30 5.74 3.27 0.56 306.73 295.02 3.22 0.00
         0.00 1573.06
         840.30 5.44 3.09 0.49 306.81 295.02 3.22 0.00
         0.00 1611.92
         836.10 5.04 2.85 0.42 306.81 296.57 3.05 0.00
         0.00 1652.86
         831.70 4.54 2.55 0.32 306.69 298.30 2.87 0.00
         0.00 1695.91
         827.20 4.14 2.31 0.25 306.73 298.30 2.87 0.00
         0.00 1740.11
         822.60 3.74 2.17 0.40 307.00 304.99 2.37 0.00
         0.00 1785.47
         817.60 3.44 2.05 0.49 307.38 306.87 1.94 0.00
         0.00 1835.00
         812.50 3.04 1.84 0.49 307.57 309.81 1.52 0.00
         0.00 1885.77
         807.00 2.54 1.54 0.40 307.61 315.00 1.38 0.00
         0.00 1940.80
         801.10 2.04 1.04 -0.13 307.27 323.13 0.97 0.00
         0.00 2000.13
         794.60 1.44 0.66 -0.28 307.28 323.13 0.97 0.10
         0.00 2065.88
         787.00 0.64 0.15 -0.46 307.18 315.00 0.82 0.10
         0.00 2143.23
         778.20 -0.16 -0.39 -0.71 307.17 315.00 0.54 0.10
         0.00 2233.47
         768.10 -0.76 -1.33 -2.07 306.54 26.57 0.87 0.10
         0.00 2338.03
         756.50 -1.66 -2.29 -3.15 306.09 45.00 1.38 0.10
         0.00 2459.47
         743.50 -2.56 -3.39 -4.60 305.58 48.81 2.06 0.20
         0.00 2597.31
         728.70 -3.56 -4.82 -6.79 304.77 32.47 2.53 0.20
         0.00 2756.60
         712.10 -4.46 -6.45 -10.03 303.87 360.00 1.55 0.10
         0.00 2938.45
         693.70 -4.66 -7.82 -14.55 303.78 310.60 1.79 0.10
         0.00 3144.54
         673.70 -6.06 -9.48 -17.85 303.55 304.99 2.37 0.10
         0.00 3374.08
         652.00 -7.46 -11.05 -21.15 303.84 336.80 2.95 0.10
         0.00 3629.57
         629.00 -8.46 -12.58 -26.99 304.48 3.81 5.85 0.20
         0.00 3908.49
         605.10 -8.86 -13.70 -36.84 306.07 11.89 11.31 0.10
         0.00 4208.48
         580.60 -9.26 -14.46 -48.61 308.56 9.67 17.35 0.10
         0.00 4528.02
         555.50 -10.56 -15.48 -46.16 311.05 4.64 21.64 0.10
         0.00 4868.57
         530.00 -12.76 -16.86 -37.11 313.30 358.53 22.73 0.10
         0.00 5228.30
         504.20 -15.76 -18.96 -33.99 314.59 355.28 23.58 0.10
         0.00 5606.56
         478.10 -19.06 -21.32 -31.63 315.80 353.80 26.96 0.10
         0.00 6004.58
         451.90 -22.26 -23.73 -30.53 317.22 352.54 32.91 0.10
         0.00 6421.27
         425.50 -25.26 -26.27 -31.32 318.83 350.81 40.15 0.00
         0.00 6860.87
         399.00 -28.56 -29.45 -35.04 319.83 348.46 46.60 0.00
         0.00 7324.52
         372.10 -32.36 -33.18 -40.23 320.57 347.33 51.36 0.00
         0.00 7820.49
         344.70 -36.66 -37.33 -45.52 321.36 349.05 55.21 0.10
         0.00 8354.86
         316.80 -41.46 -41.91 -49.66 322.29 352.90 61.27 0.10
         0.00 8933.26
         289.20 -46.66 -46.90 -52.70 323.26 353.40 70.98 0.10
         0.00 9544.55
         263.10 -51.76 -51.92 -57.69 324.47 350.76 81.08 0.10
         0.00 10164.59
         239.40 -56.86 -56.94 -61.61 325.58 349.20 85.02 0.10
         0.00 10769.30
         218.00 -62.06 -62.12 -67.47 326.29 349.79 81.12 0.10
         0.00 11355.03
         198.70 -65.46 -65.51 -72.95 329.60 352.75 64.63 0.10
         0.00 11923.20
         181.30 -65.56 -65.62 -73.57 338.19 345.85 47.67 0.00
         0.00 12480.21
         165.60 -63.86 -63.95 -74.17 349.89 334.77 41.01 0.00
         0.00 13032.85
         151.10 -61.16 -9999.00 -9999.00 -9999.00 334.11 36.93 0.00
         0.00 13597.84
         137.30 -59.76 -9999.00 -9999.00 -9999.00 329.66 30.38 0.00
         0.00 14194.10
         124.00 -59.06 -9999.00 -9999.00 -9999.00 331.07 25.29 0.00
         0.00 14831.55
         111.30 -58.46 -9999.00 -9999.00 -9999.00 328.24 19.19 0.00
         0.00 15509.64
         98.90 -57.86 -9999.00 -9999.00 -9999.00 317.01 15.68 0.00
         0.00 16252.97
         86.90 -57.76 -9999.00 -9999.00 -9999.00 308.16 13.83 0.00
         0.00 17068.31
         75.10 -58.36 -9999.00 -9999.00 -9999.00 307.63 11.77 0.00
         0.00 17987.13
         63.60 -58.26 -58.80 -80.31 472.23 316.85 8.53 0.00
         0.00 19032.36
         52.30 -58.16 -9999.00 -9999.00 -9999.00 328.24 4.80 0.00
         0.00 20263.10
         41.00 -58.66 -59.44 -82.99 534.34 12.53 5.38 0.00
         0.00 21793.21
         29.80 -58.16 -59.27 -84.87 586.71 66.61 7.83 0.00
         0.00 23798.77
         18.70 -56.96 -58.85 -87.55 674.01 97.97 9.81 0.00
         0.00 26739.45
         7.60 -48.76 -55.67 -92.48 904.81 303.69 6.29 0.00
         0.00 32545.28"
    }

    #[test]
    fn test_parse() {
        use chrono::NaiveDate;
        use optional::some;

        let test_data = get_test_data();

        let snd = UpperAir::parse(test_data);
        assert!(snd.is_ok());

        let snd = snd.unwrap();

        assert_eq!(snd.num, 727730);
        assert_eq!(
            snd.valid_time,
            NaiveDate::from_ymd_opt(2017, 4, 1).unwrap().and_hms_opt(1, 0, 0).unwrap()
        );
        assert_eq!(snd.lead_time, 1);
        assert_eq!(snd.lat, some(46.87));
        assert_eq!(snd.lon, some(-114.16));
        assert_eq!(snd.elevation, some(Meters(1335.0)));

        assert_eq!(snd.show, some(CelsiusDiff(8.12)));
        assert_eq!(snd.li, some(CelsiusDiff(8.0)));
        assert_eq!(snd.swet, some(39.08));
        assert_eq!(snd.kinx, some(Celsius(14.88)));
        assert_eq!(snd.lclp, some(HectoPascal(780.77)));
        assert_eq!(snd.pwat, some(Mm(9.28)));
        assert_eq!(snd.totl, some(39.55));
        assert_eq!(snd.cape, some(JpKg(0.0)));
        assert_eq!(snd.lclt, some(Kelvin(272.88)));
        assert_eq!(snd.cins, some(JpKg(0.0)));
        assert!(snd.eqlv.is_none());
        assert!(snd.lfc.is_none());
        assert_eq!(snd.brch, some(0.0));

        // Upper air - 3rd level.
        assert_eq!(snd.pressure[2], some(HectoPascal(859.8)));
        assert_eq!(snd.temperature[2], some(Celsius(7.24)));
        assert_eq!(snd.wet_bulb[2], some(Celsius(4.18)));
        assert_eq!(snd.dew_point[2], some(Celsius(0.90)));
        assert_eq!(snd.theta_e[2], some(Kelvin(306.87)));
        assert_eq!(
            snd.wind[2],
            some(WindSpdDir {
                direction: 292.38,
                speed: Knots(3.57)
            })
        );
        assert_eq!(snd.omega[2], some(PaPS(0.00)));
        assert_eq!(snd.height[2], some(Meters(1423.71)));
        assert_eq!(snd.cloud_fraction[2], some(0.0));

        assert_eq!(snd.pressure.len(), 60);
        assert_eq!(snd.temperature.len(), 60);
        assert_eq!(snd.wet_bulb.len(), 60);
        assert_eq!(snd.dew_point.len(), 60);
        assert_eq!(snd.theta_e.len(), 60);
        assert_eq!(snd.wind.len(), 60);
        assert_eq!(snd.omega.len(), 60);
        assert_eq!(snd.height.len(), 60);
        assert_eq!(snd.cloud_fraction.len(), 60);
    }
}
