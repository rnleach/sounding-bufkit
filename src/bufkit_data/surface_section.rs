//! Deals with the text and parsing of the surface section in a bufkit file.

use crate::bufkit_data::surface::{SfcColumns, SurfaceData};
use crate::error::*;
use std::error::Error;

/// Represents the section of a string that represents surface data in a bufkit file.
pub struct SurfaceSection<'a> {
    raw_text: &'a str,
    columns: SfcColumns,
}

impl<'a> SurfaceSection<'a> {
    /// Create a new SurfaceSection.
    pub fn new(text: &'a str) -> Result<SurfaceSection<'a>, BufkitFileError> {
        // Split the header off
        let mut header_end: usize = 0;
        let mut previous_char = 'x';
        let mut found = false;
        for (i, c) in text.char_indices() {
            header_end = i;
            if previous_char.is_whitespace() && c.is_digit(10) {
                found = true;
                break;
            } else {
                previous_char = c;
            }
        }
        if !found {
            return Err(BufkitFileError::new());
        }
        let header = &text[0..header_end].trim();

        // Parse the column headers
        let cols = SurfaceData::parse_columns(header)?;

        Ok(SurfaceSection {
            raw_text: text[header_end..].trim(),
            columns: cols,
        })
    }

    /// Validate the surface section of a sounding.
    pub fn validate_section(&self) -> Result<(), Box<dyn Error>> {
        let mut iter = self.into_iter();

        loop {
            let opt = iter.get_next_chunk()?;
            if let Some(chunk) = opt {
                SurfaceData::parse_values(chunk, iter.columns)?;
            } else {
                break;
            }
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a SurfaceSection<'a> {
    type Item = SurfaceData;
    type IntoIter = SurfaceIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SurfaceIterator {
            remaining: self.raw_text,
            columns: &self.columns,
        }
    }
}

/// Iterator struct that parses one entry at a time.
///
/// If there is a parsing error, it skips the entry that caused it and moves on.
pub struct SurfaceIterator<'a> {
    remaining: &'a str,
    columns: &'a SfcColumns,
}

impl<'a> SurfaceIterator<'a> {
    fn get_next_chunk(&mut self) -> Result<Option<&'a str>, BufkitFileError> {
        use crate::parse_util::find_next_n_tokens;
        if let Some(brk) = find_next_n_tokens(self.remaining, self.columns.num_cols())? {
            let next_chunk = &self.remaining[0..brk];
            self.remaining = &self.remaining[brk..];
            Ok(Some(next_chunk))
        } else {
            Ok(None)
        }
    }
}

impl<'a> Iterator for SurfaceIterator<'a> {
    type Item = SurfaceData;

    fn next(&mut self) -> Option<SurfaceData> {
        while let Ok(Some(text)) = self.get_next_chunk() {
            if let Ok(sd) = SurfaceData::parse_values(text, self.columns) {
                return Some(sd);
            }
        }
        // Ran out of text to try.
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_valid_test_data() -> &'static str {
        "
        STN YYMMDD/HHMM PMSL PRES SKTC STC1 EVAP P03M
        C03M SWEM LCLD MCLD HCLD UWND
        VWND T2MS Q2MS WXTS WXTP WXTZ
        WXTR S03M TD2M
        727730 170401/0000 1020.40 909.10 10.54 278.70 -9999.00 0.00
        0.00 0.00 100.00 0.00 58.00 0.90
        -0.10 10.34 4.59 0.00 0.00 0.00
        0.00 -9999.00 1.13
        727730 170401/0300 1021.50 909.40 2.14 278.20 10.00 0.00
        0.00 0.00 52.00 0.00 61.00 1.30
        0.50 3.24 4.42 0.00 0.00 0.00
        0.00 -9999.00 0.62
        727730 170401/0600 1022.30 909.40 0.34 277.50 -9999.00 0.00
        0.00 0.00 2.00 0.00 39.00 1.10
        0.60 1.24 3.91 0.00 0.00 0.00
        0.00 -9999.00 -1.06
        727730 170401/0900 1022.70 909.30 -0.66 277.00 -9999.00 0.00
        0.00 0.02 1.00 0.00 33.00 1.10
        0.60 0.24 3.65 0.00 0.00 0.00
        0.00 -9999.00 -1.99
        727730 170401/1200 1022.60 908.80 -1.16 276.80 -9999.00 0.00
        0.00 0.02 3.00 0.00 49.00 0.60
        0.80 -0.56 3.50 0.00 0.00 0.00
        0.00 -9999.00 -2.56
        727730 170401/1500 1021.80 908.60 3.44 276.20 2.00 0.00
        0.00 0.02 4.00 0.00 77.00 0.40
        0.60 2.84 3.88 0.00 0.00 0.00
        0.00 -9999.00 -1.17"
    }

    fn get_invalid_test_data1() -> &'static str {
        // Missing last value from last surface observation.
        "
        STN YYMMDD/HHMM PMSL PRES SKTC STC1 EVAP P03M
        C03M SWEM LCLD MCLD HCLD UWND
        VWND T2MS Q2MS WXTS WXTP WXTZ
        WXTR S03M TD2M
        727730 170401/0000 1020.40 909.10 10.54 278.70 -9999.00 0.00
        0.00 0.00 100.00 0.00 58.00 0.90
        -0.10 10.34 4.59 0.00 0.00 0.00
        0.00 -9999.00 1.13
        727730 170401/0300 1021.50 909.40 2.14 278.20 10.00 0.00
        0.00 0.00 52.00 0.00 61.00 1.30
        0.50 3.24 4.42 0.00 0.00 0.00
        0.00 -9999.00 0.62
        727730 170401/0600 1022.30 909.40 0.34 277.50 -9999.00 0.00
        0.00 0.00 2.00 0.00 39.00 1.10
        0.60 1.24 3.91 0.00 0.00 0.00
        0.00 -9999.00 -1.06
        727730 170401/0900 1022.70 909.30 -0.66 277.00 -9999.00 0.00
        0.00 0.02 1.00 0.00 33.00 1.10
        0.60 0.24 3.65 0.00 0.00 0.00
        0.00 -9999.00 -1.99
        727730 170401/1200 1022.60 908.80 -1.16 276.80 -9999.00 0.00
        0.00 0.02 3.00 0.00 49.00 0.60
        0.80 -0.56 3.50 0.00 0.00 0.00
        0.00 -9999.00 -2.56
        727730 170401/1500 1021.80 908.60 3.44 276.20 2.00 0.00
        0.00 0.02 4.00 0.00 77.00 0.40
        0.60 2.84 3.88 0.00 0.00 0.00
        0.00 -9999.00"
    }

    fn get_invalid_test_data2() -> &'static str {
        // Invalid columns
        "
        STN PMSL PRES SKTC STC1 EVAP P03M
        C03M SWEM LCLD MCLD HCLD UWND
        VWND T2MS Q2MS WXTS WXTP WXTZ
        WXTR S03M TD2M
        727730 170401/0000 1020.40 909.10 10.54 278.70 -9999.00 0.00
        0.00 0.00 100.00 0.00 58.00 0.90
        -0.10 10.34 4.59 0.00 0.00 0.00
        0.00 -9999.00 1.13
        727730 170401/0300 1021.50 909.40 2.14 278.20 10.00 0.00
        0.00 0.00 52.00 0.00 61.00 1.30
        0.50 3.24 4.42 0.00 0.00 0.00
        0.00 -9999.00 0.62
        727730 170401/0600 1022.30 909.40 0.34 277.50 -9999.00 0.00
        0.00 0.00 2.00 0.00 39.00 1.10
        0.60 1.24 3.91 0.00 0.00 0.00
        0.00 -9999.00 -1.06
        727730 170401/0900 1022.70 909.30 -0.66 277.00 -9999.00 0.00
        0.00 0.02 1.00 0.00 33.00 1.10
        0.60 0.24 3.65 0.00 0.00 0.00
        0.00 -9999.00 -1.99
        727730 170401/1200 1022.60 908.80 -1.16 276.80 -9999.00 0.00
        0.00 0.02 3.00 0.00 49.00 0.60
        0.80 -0.56 3.50 0.00 0.00 0.00
        0.00 -9999.00 -2.56
        727730 170401/1500 1021.80 908.60 3.44 276.20 2.00 0.00
        0.00 0.02 4.00 0.00 77.00 0.40
        0.60 2.84 3.88 0.00 0.00 0.00
        0.00 -9999.00 -1.17"
    }

    #[test]
    fn test_surface_through_iterator() {
        use chrono::{NaiveDate, NaiveDateTime};
        let test_data = get_valid_test_data();

        let surface_section = SurfaceSection::new(test_data).unwrap();

        assert_eq!(surface_section.into_iter().count(), 6);

        for sd in &surface_section {
            assert_eq!(sd.station_num, 727730);
        }

        assert_eq!(
            surface_section
                .into_iter()
                .map(|sd| sd.valid_time)
                .collect::<Vec<NaiveDateTime>>(),
            vec![
                NaiveDate::from_ymd(2017, 4, 1).and_hms(0, 0, 0),
                NaiveDate::from_ymd(2017, 4, 1).and_hms(3, 0, 0),
                NaiveDate::from_ymd(2017, 4, 1).and_hms(6, 0, 0),
                NaiveDate::from_ymd(2017, 4, 1).and_hms(9, 0, 0),
                NaiveDate::from_ymd(2017, 4, 1).and_hms(12, 0, 0),
                NaiveDate::from_ymd(2017, 4, 1).and_hms(15, 0, 0),
            ]
        );

        assert_eq!(
            surface_section
                .into_iter()
                .map(|sd| sd.mslp)
                .collect::<Vec<f64>>(),
            vec![1020.4, 1021.5, 1022.3, 1022.7, 1022.6, 1021.8]
        );
        assert_eq!(
            surface_section
                .into_iter()
                .map(|sd| sd.station_pres)
                .collect::<Vec<f64>>(),
            vec![909.1, 909.4, 909.4, 909.3, 908.8, 908.6]
        );
        assert_eq!(
            surface_section
                .into_iter()
                .map(|sd| sd.low_cloud)
                .collect::<Vec<f64>>(),
            vec![100.0, 52.0, 2.0, 1.0, 3.0, 4.0]
        );
        assert_eq!(
            surface_section
                .into_iter()
                .map(|sd| sd.mid_cloud)
                .collect::<Vec<f64>>(),
            vec![0.0; 6]
        );
        assert_eq!(
            surface_section
                .into_iter()
                .map(|sd| sd.hi_cloud)
                .collect::<Vec<f64>>(),
            vec![58.0, 61.0, 39.0, 33.0, 49.0, 77.0]
        );
        assert_eq!(
            surface_section
                .into_iter()
                .map(|sd| sd.uwind)
                .collect::<Vec<f64>>(),
            vec![0.9, 1.3, 1.1, 1.1, 0.6, 0.4]
        );
        assert_eq!(
            surface_section
                .into_iter()
                .map(|sd| sd.vwind)
                .collect::<Vec<f64>>(),
            vec![-0.1, 0.5, 0.6, 0.6, 0.8, 0.6]
        );
    }

    #[test]
    fn test_validate() {
        let surface_section = SurfaceSection::new(get_valid_test_data()).unwrap();
        assert!(surface_section.validate_section().is_ok());

        println!("DOING TEST 1");
        let surface_section = SurfaceSection::new(get_invalid_test_data1()).unwrap();
        assert!(!surface_section.validate_section().is_ok());
        println!("DONE TEST 1");

        assert!(SurfaceSection::new(get_invalid_test_data2()).is_err());
    }
}
