//! Parse the station info section of a bufkit upper air section.

use chrono::NaiveDateTime;

use error::*;

/// Information related to the geographic location of the sounding.
#[derive(Debug)]
pub struct StationInfo {
    pub num: i32,                  // station number, USAF number, eg 727730
    pub valid_time: NaiveDateTime, // valid time of sounding
    pub lead_time: i32,            // Forecast lead time in hours from model initialization
    pub lat: f64,                  // latitude
    pub lon: f64,                  // longitude
    pub elevation: f64,            // elevation (m)
}

impl StationInfo {
    /// Given a String or slice of characters, parse them into a StationInfo struct.
    pub fn parse(src: &str) -> Result<StationInfo, Error> {
        // This method assumes that these values are ALWAYS in this order. If it turns out that
        // they are not, it will probably error! The easy fix would be to replace head with src
        // in all of the parse_* function calls below, at the expense of a probably slower parsing
        // function.
        //
        // STID - Station ID (alphanumeric)
        // STNM - 6-digit station Number
        // TIME - Valid time (UTC) in YYMMDD/HHMM numeric format
        // SLAT - Latitude (decimal degrees)
        // SLON - Longitude (decimal degrees)
        // SELV - Station elevation (m)
        // STIM - Forecast hour

        use parse_util::{parse_kv, parse_naive_date_time, parse_f64, parse_i32};

        // Get station num
        let (station_num, head) = parse_i32(src, "STNM")?;

        // Get valid time
        let (val_to_parse, head) = parse_kv(
            head,
            "TIME",
            |c| char::is_digit(c, 10),
            |c| !(char::is_digit(c, 10) || c == '/'),
        )?;

        let vt = parse_naive_date_time(val_to_parse)?;

        // get latitude, longitude, and elevation
        let (lat, head) = parse_f64(head, "SLAT")?;

        let (lon, head) = parse_f64(head, "SLON")?;

        let (elv, head) = parse_f64(head, "SELV")?;

        // get the lead time
        let (lt, _) = parse_i32(head, "STIM")?;

        Ok(StationInfo {
            num: station_num,
            valid_time: vt,
            lead_time: lt,
            lat,
            lon,
            elevation: elv,
        })
    }
}

#[test]
fn test_station_info_parse() {
    use chrono::NaiveDate;

    let test_data = "STID = STNM = 727730 TIME = 170401/0000
                     SLAT = 46.92 SLON = -114.08 SELV = 972.0
                     STIM = 0";

    let si = StationInfo::parse(&test_data);
    println!("si: {:?}", si);

    let StationInfo {
        num,
        valid_time,
        lead_time,
        lat,
        lon,
        elevation,
    } = si.unwrap();
    assert_eq!(num, 727730);
    assert_eq!(valid_time, NaiveDate::from_ymd(2017, 4, 1).and_hms(0, 0, 0));
    assert_eq!(lead_time, 0);
    assert_eq!(lat, 46.92);
    assert_eq!(lon, -114.08);
    assert_eq!(elevation, 972.0);

    let test_data = "STID = KMSO STNM = 727730 TIME = 170404/1200
                     SLAT = 46.87 SLON = -114.16 SELV = 1335.0
                     STIM = 84";

    let si = StationInfo::parse(&test_data);
    println!("si: {:?}", si);

    let StationInfo {
        num,
        valid_time,
        lead_time,
        lat,
        lon,
        elevation,
    } = si.unwrap();
    assert_eq!(num, 727730);
    assert_eq!(
        valid_time,
        NaiveDate::from_ymd(2017, 4, 4).and_hms(12, 0, 0)
    );
    assert_eq!(lead_time, 84);
    assert_eq!(lat, 46.87);
    assert_eq!(lon, -114.16);
    assert_eq!(elevation, 1335.0);
}
