//! Parses the string representing the upper air indexes from a bufkit file.

use error::*;

/// Several stability indexes.
#[derive(Debug)]
pub struct Indexes {
    pub show: f64, // Showalter index
    pub li: f64,   // Lifted index
    pub swet: f64, // Severe Weather Threat index
    pub kinx: f64, // K-index
    pub lclp: f64, // Lifting Condensation Level (hPa)
    pub pwat: f64, // Precipitable water (mm)
    pub totl: f64, // Total-Totals
    pub cape: f64, // Convective Available Potential Energy
    pub lclt: f64, // Temperature at LCL (K)
    pub cins: f64, // Convective Inhibitive Energy
    pub eqlv: f64, // Equilibrium Level (hPa)
    pub lfc: f64,  // Level of Free Convection (hPa)
    pub brch: f64, // Bulk Richardson Number
}

impl Indexes {
    pub fn parse(src: &str) -> Result<Indexes> {
        // This method assumes that these values are ALWAYS in this order. If it turns out that
        // they are not, it will probably error by using a default value, which is the missing
        // value! The easy fix would be to replace head with src in all of the parse_f64 function
        // calls below, at the expense of a probably slower parsing function.
        //
        // SHOW - Showalter Index
        // LIFT - Lifted Index
        // SWET - SWET Index
        // KINX - K Index
        // LCLP - Pressure at the LCL (hPa)
        // PWAT - Precipitable water (mm)
        // TOTL - Total Totals Index
        // CAPE - CAPE (Convective Available Potential Energy)
        // LCLT - Temperature at the LCL (K)
        // CINS - CINS (Convective Inhibition)
        // EQLV - Equilibrium level (hPa)
        // LFCT - Level of free convection (hPa)
        // BRCH - Bulk Richardson number

        use parse_util::parse_f64;

        let (show, head) = parse_f64(src, "SHOW").unwrap_or((-9999.0, src));
        let (lift, head) = parse_f64(head, "LIFT").unwrap_or((-9999.0, head));
        let (swet, head) = parse_f64(head, "SWET").unwrap_or((-9999.0, head));
        let (kinx, head) = parse_f64(head, "KINX").unwrap_or((-9999.0, head));
        let (lclp, head) = parse_f64(head, "LCLP").unwrap_or((-9999.0, head));
        let (pwat, head) = parse_f64(head, "PWAT").unwrap_or((-9999.0, head));
        let (totl, head) = parse_f64(head, "TOTL").unwrap_or((-9999.0, head));
        let (cape, head) = parse_f64(head, "CAPE").unwrap_or((-9999.0, head));
        let (lclt, head) = parse_f64(head, "LCLT").unwrap_or((-9999.0, head));
        let (cins, head) = parse_f64(head, "CINS").unwrap_or((-9999.0, head));
        let (eqlv, head) = parse_f64(head, "EQLV").unwrap_or((-9999.0, head));
        let (lfct, head) = parse_f64(head, "LFCT").unwrap_or((-9999.0, head));
        let (brch, _) = parse_f64(head, "BRCH").unwrap_or((-9999.0, head));

        Ok(Indexes {
            show: show,
            li: lift,
            swet: swet,
            kinx: kinx,
            lclp: lclp,
            pwat: pwat,
            totl: totl,
            cape: cape,
            lclt: lclt,
            cins: cins,
            eqlv: eqlv,
            lfc: lfct,
            brch: brch,
        })
    }
}

#[test]
fn test_indexes_parse() {
    let test_data = "
        SHOW = 8.12 LIFT = 8.00 SWET = 39.08 KINX = 14.88
        LCLP = 780.77 PWAT = 9.28 TOTL = 39.55 CAPE = 0.00
        LCLT = 272.88 CINS = 0.00 EQLV = -9999.00 LFCT = -9999.00
        BRCH = 0.00";

    let indexes = Indexes::parse(&test_data);
    println!("indexes: {:?}", indexes);

    let Indexes {
        show,
        li,
        swet,
        kinx,
        lclp,
        pwat,
        totl,
        cape,
        lclt,
        cins,
        eqlv,
        lfc,
        brch,
    } = indexes.unwrap();

    assert_eq!(show, 8.12);
    assert_eq!(li, 8.0);
    assert_eq!(swet, 39.08);
    assert_eq!(kinx, 14.88);
    assert_eq!(lclp, 780.77);
    assert_eq!(pwat, 9.28);
    assert_eq!(totl, 39.55);
    assert_eq!(cape, 0.00);
    assert_eq!(lclt, 272.88);
    assert_eq!(cins, 0.00);
    assert_eq!(eqlv, -9999.0);
    assert_eq!(lfc, -9999.0);
    assert_eq!(brch, 0.00);

    let test_data = "
        SHOW = 9.67 LIFT = 9.84 SWET = 33.41 KINX = 3.88
        LCLP = 822.95 PWAT = 9.52 TOTL = 37.25 CAPE = 0.00
        LCLT = 273.49 CINS = 0.00 EQLV = -9999.00 LFCT = -9999.00
        BRCH = 0.00";

    let indexes = Indexes::parse(&test_data);
    println!("indexes: {:?}", indexes);

    let Indexes {
        show,
        li,
        swet,
        kinx,
        lclp,
        pwat,
        totl,
        cape,
        lclt,
        cins,
        eqlv,
        lfc,
        brch,
    } = indexes.unwrap();

    assert_eq!(show, 9.67);
    assert_eq!(li, 9.84);
    assert_eq!(swet, 33.41);
    assert_eq!(kinx, 3.88);
    assert_eq!(lclp, 822.95);
    assert_eq!(pwat, 9.52);
    assert_eq!(totl, 37.25);
    assert_eq!(cape, 0.00);
    assert_eq!(lclt, 273.49);
    assert_eq!(cins, 0.00);
    assert_eq!(eqlv, -9999.0);
    assert_eq!(lfc, -9999.0);
    assert_eq!(brch, 0.00);
}
