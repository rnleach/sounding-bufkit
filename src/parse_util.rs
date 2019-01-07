//! Utilites for parsing a sounding.
use std::error::Error;

use chrono::{NaiveDate, NaiveDateTime};
use crate::error::*;

/// Isolate a value into a sub-string for further parsing.
///
/// Given a string `src` with a sub-string of the form "KEY = VALUE" (with or without spaces), and
/// closures that describe the first character you want to keep after the '=' and the last
/// character in the sub-string you want to keep, return a tuple with the first value as the
/// sub-string you were looking for and the second value the remainder of `src` after this
/// sub-string has been parsed out.
pub fn parse_kv<'a, 'b, FS, FE>(
    src: &'a str,
    key: &'b str,
    start_val: FS,
    end_val: FE,
) -> Result<(&'a str, &'a str), BufkitFileError>
where
    FS: Fn(char) -> bool,
    FE: Fn(char) -> bool,
{
    let mut idx = src.find(key).ok_or_else(BufkitFileError::new)?;
    let mut head = &src[idx..];
    idx = head.find(start_val).ok_or_else(BufkitFileError::new)?;
    head = &head[idx..];
    // When finding the end of the value, you may go all the way to the end of the slice.
    // If so, find returns None, just convert that into the end of the slice.
    let tail_idx = head.find(end_val).or_else(|| Some(head.len())).unwrap();
    Ok((head[..tail_idx].trim(), &head[tail_idx..]))
}

#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn test_parse_kv() {
    let test_data =
        "STID = STNM = 727730 TIME = 170401/0000 \
         SLAT = 46.92 SLON = -114.08 SELV = 972.0 \
         STIM = 0";

    if let Ok((value_to_parse, head)) =
        parse_kv(test_data,
                 "STNM",
                 |c| char::is_digit(c, 10),
                 |c| !char::is_digit(c, 10)) {
        assert_eq!(value_to_parse, "727730");
        assert_eq!(head, " TIME = 170401/0000 SLAT = 46.92 SLON = -114.08 SELV = 972.0 STIM = 0");
    } else {
        assert!(false, "There was an error parsing.");
    }

    if let Ok((val_to_parse, head)) =
        parse_kv(test_data,
                 "TIME",
                 |c| char::is_digit(c, 10),
                 |c| !(char::is_digit(c, 10) || c == '/')) {
        assert_eq!(val_to_parse, "170401/0000");
        assert_eq!(head, " SLAT = 46.92 SLON = -114.08 SELV = 972.0 STIM = 0");
    } else {
        assert!(false, "There was an error parsing.");
    }

    if let Ok((val_to_parse, head)) =
        parse_kv(test_data,
                 "STIM",
                 |c| char::is_digit(c, 10),
                 |c| !char::is_digit(c, 10)) {
        assert_eq!(val_to_parse, "0");
        assert_eq!(head, "");
    } else {
        assert!(false, "There was an error parsing the very last element.");
    }
}

/// Parse an f64 value.
pub fn parse_f64<'a, 'b>(src: &'a str, key: &'b str) -> Result<(f64, &'a str), Box<dyn Error>> {
    use std::str::FromStr;

    let (val_to_parse, head) = parse_kv(
        src,
        key,
        |c| char::is_digit(c, 10) || c == '-',
        |c| !(char::is_digit(c, 10) || c == '.' || c == '-'),
    )?;
    let val = f64::from_str(val_to_parse)?;
    Ok((val, head))
}

#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn test_parse_f64() {
    let test_data =
        "STID = STNM = 727730 TIME = 170401/0000 \
         SLAT = 46.92 SLON = -114.08 SELV = 972.0 \
         STIM = 0";

    if let Ok((lat, head)) = parse_f64(test_data, "SLAT") {
        assert_eq!(lat, 46.92);
        assert_eq!(head, " SLON = -114.08 SELV = 972.0 STIM = 0");
    } else {
        assert!(false, "There was an error parsing.");
    }

    if let Ok((lon, head)) = parse_f64(test_data, "SLON") {
        assert_eq!(lon, -114.08);
        assert_eq!(head, " SELV = 972.0 STIM = 0");
    } else {
        assert!(false, "There was an error parsing.");
    }
}

/// Parse an i32 value.
pub fn parse_i32<'a, 'b>(src: &'a str, key: &'b str) -> Result<(i32, &'a str), Box<dyn Error>> {
    use std::str::FromStr;

    let (val_to_parse, head) = parse_kv(
        src,
        key,
        |c| char::is_digit(c, 10),
        |c| !char::is_digit(c, 10),
    )?;
    let val = i32::from_str(val_to_parse)?;
    Ok((val, head))
}

#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn test_parse_i32() {
    let test_data =
        "STID = STNM = 727730 TIME = 170401/0000 \
         SLAT = 46.92 SLON = -114.08 SELV = 972.0 \
         STIM = 0";

    if let Ok((stnm, head)) = parse_i32(test_data, "STNM") {
        assert_eq!(stnm, 727730);
        assert_eq!(head, " TIME = 170401/0000 SLAT = 46.92 SLON = -114.08 SELV = 972.0 STIM = 0");
    } else {
        assert!(false, "There was an error parsing.");
    }

    if let Ok((ymd, head)) = parse_i32(test_data, "TIME") {
        assert_eq!(ymd, 170401);
        assert_eq!(head, "/0000 SLAT = 46.92 SLON = -114.08 SELV = 972.0 STIM = 0");
    } else {
        assert!(false, "There was an error parsing.");
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(doc_markdown))]
/// Parse a string of the form "YYmmdd/hhMM" to a `NaiveDateTime`.
pub fn parse_naive_date_time(src: &str) -> Result<NaiveDateTime, Box<dyn Error>> {
    use std::str::FromStr;

    let val_to_parse = src.trim();

    let year = i32::from_str(&val_to_parse[..2])? + 2000;
    let month = u32::from_str(&val_to_parse[2..4])?;
    let day = u32::from_str(&val_to_parse[4..6])?;
    let hour = u32::from_str(&val_to_parse[7..9])?;
    let minute = u32::from_str(&val_to_parse[9..11])?;
    Ok(NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, 0))
}

#[test]
fn test_parse_naive_date_time() {
    let test_data = " 170401/0000 ";

    let test_value = parse_naive_date_time(test_data).unwrap();
    assert_eq!(test_value, NaiveDate::from_ymd(2017, 4, 1).and_hms(0, 0, 0));
}

/// Find a blank line, or a line without any ASCII numbers or letters.
///
/// Return `None` if one cannot be found, otherwise return the byte location of the character just
/// after the second newline.
pub fn find_blank_line(src: &str) -> Option<usize> {
    let mut first_newline = false;

    let mut iter = src.char_indices().peekable();
    loop {
        let (_, c) = iter.next()?;

        if c == '\n' && !first_newline {
            first_newline = true;
        } else if c.is_alphanumeric() {
            // Found a letter or number, since last newline, reset flag.
            first_newline = false;
        } else if c == '\n' && first_newline {
            // We've found the second one in a row!
            if let Some(&(next_index, _)) = iter.peek() {
                return Some(next_index);
            } else {
                return None;
            }
        }
    }
}

#[test]
fn test_find_blank_line() {
    let test_string = "STID = STNM = 727730 TIME = 170401/0300
                       SLAT = 46.92 SLON = -114.08 SELV = 972.0
                       STIM = 3

                       SHOW = 9.67 LIFT = 9.84 SWET = 33.41 KINX = 3.88
                       LCLP = 822.95 PWAT = 9.52 TOTL = 37.25 CAPE = 0.00
                       LCLT = 273.49 CINS = 0.00 EQLV = -9999.00 LFCT = -9999.00
                       BRCH = 0.00

                       PRES TMPC TMWC DWPC THTE DRCT SKNT OMEG
                       HGHT
                       906.90 8.04 4.99 1.70 303.11 250.71 4.12 -2.00";

    let (station_info, the_rest) = test_string.split_at(find_blank_line(test_string).unwrap());
    let (indexes, the_rest) = the_rest.split_at(find_blank_line(the_rest).unwrap());

    assert!(station_info.trim().starts_with("STID = STNM = 727730"));
    assert!(station_info.trim().ends_with("STIM = 3"));

    assert!(indexes.trim().starts_with("SHOW = 9.67"));
    assert!(indexes.trim().ends_with("BRCH = 0.00"));

    assert!(the_rest.trim().starts_with("PRES TMPC TMWC"));
    assert!(find_blank_line(the_rest).is_none());
}

/// In a list of white space delimited floating point values, find a string with `n` values.
pub fn find_next_n_tokens(src: &str, n: usize) -> Result<Option<usize>, BufkitFileError> {
    if src.trim().is_empty() {
        return Ok(None);
    }

    let mut started = false;
    let mut token_count = 0;
    let mut in_white_space = src.starts_with(char::is_whitespace);

    for (i, c) in src.char_indices() {
        if !started && (c.is_numeric() || c == '-' || c == '.') {
            started = true;
        } else if !in_white_space && c.is_whitespace() {
            // Just passed into white space, increase token count
            token_count += 1;
            in_white_space = true;
        } else if in_white_space && !c.is_whitespace() {
            // Just passed out of white space
            in_white_space = false;
        }

        if token_count == n {
            return Ok(Some(i));
        }
    }

    // Special case for end of string
    if !in_white_space && token_count == n - 1 {
        return Ok(Some(src.len()));
    }

    // Invalid number of tokens
    if token_count > 0 {
        return Err(BufkitFileError::new());
    }
    // Out of tokens
    Ok(None)
}

#[test]
fn test_find_next_n_tokens() {
    let test_data = "
        727730 170401/0700 1021.50 869.80 0.14 275.50 0.00 74.00
        0.00 0.00 277.40 0.00 0.00 0.00
        0.00 1.00 0.70 0.00 0.07 1.44
        3.73 0.00 0.00 0.00 0.00 -4.60
        -4.80 30.30 0.01 999.00 -9999.00 20.00
        -2.30
        727730 170401/0800 1022.00 869.70 -0.36 274.90 0.00 74.00
        0.00 0.00 277.20 0.00 0.00 0.00
        0.00 1.00 0.50 0.00 0.07 0.34
        3.60 0.00 0.00 0.00 0.00 -3.70
        -5.30 35.40 0.01 999.00 -9999.00 20.00
        -2.78
        727730 170401/0900 1022.80 869.80 -0.46 274.80 0.00 74.00
        0.00 0.00 277.10 0.00 0.00 0.00
        0.00 0.90 0.80 0.00 0.07 -0.56
        3.50 0.00 0.00 0.00 0.00 -2.70
        -6.70 31.90 0.01 999.00 -9999.00 20.00
        -3.15";

    let brk = find_next_n_tokens(test_data, 33).unwrap().unwrap();
    let (substr, remaining) = test_data.split_at(brk);

    println!("First: {}", substr);
    assert_eq!(
        substr,
        "
        727730 170401/0700 1021.50 869.80 0.14 275.50 0.00 74.00
        0.00 0.00 277.40 0.00 0.00 0.00
        0.00 1.00 0.70 0.00 0.07 1.44
        3.73 0.00 0.00 0.00 0.00 -4.60
        -4.80 30.30 0.01 999.00 -9999.00 20.00
        -2.30"
    );

    let brk = find_next_n_tokens(remaining, 33).unwrap().unwrap();
    let (substr, remaining) = remaining.split_at(brk);
    println!("Second: {}", substr);
    assert_eq!(
        substr,
        "
        727730 170401/0800 1022.00 869.70 -0.36 274.90 0.00 74.00
        0.00 0.00 277.20 0.00 0.00 0.00
        0.00 1.00 0.50 0.00 0.07 0.34
        3.60 0.00 0.00 0.00 0.00 -3.70
        -5.30 35.40 0.01 999.00 -9999.00 20.00
        -2.78"
    );

    let brk = find_next_n_tokens(remaining, 33).unwrap().unwrap();
    let (substr, remaining) = remaining.split_at(brk);
    println!("Third: {}", substr.trim());
    assert_eq!(
        substr,
        "
        727730 170401/0900 1022.80 869.80 -0.46 274.80 0.00 74.00
        0.00 0.00 277.10 0.00 0.00 0.00
        0.00 0.90 0.80 0.00 0.07 -0.56
        3.50 0.00 0.00 0.00 0.00 -2.70
        -6.70 31.90 0.01 999.00 -9999.00 20.00
        -3.15"
    );

    assert_eq!(find_next_n_tokens(remaining, 33).unwrap(), None);
}
