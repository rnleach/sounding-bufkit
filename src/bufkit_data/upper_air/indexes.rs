//! Parses the string representing the upper air indexes from a bufkit file.

use crate::error::*;
use metfor::{Celsius, CelsiusDiff, HectoPascal, JpKg, Kelvin, Mm};
use optional::{none, Optioned};

/// Several stability indexes.
#[derive(Debug)]
pub struct Indexes {
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
}

impl Indexes {
    pub fn parse(src: &str) -> Result<Indexes, BufkitFileError> {
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

        use crate::parse_util::parse_f64;

        let (show, head) = parse_f64(src, "SHOW").unwrap_or((none(), src));
        let (lift, head) = parse_f64(head, "LIFT").unwrap_or((none(), head));
        let (swet, head) = parse_f64(head, "SWET").unwrap_or((none(), head));
        let (kinx, head) = parse_f64(head, "KINX").unwrap_or((none(), head));
        let (lclp, head) = parse_f64(head, "LCLP").unwrap_or((none(), head));
        let (pwat, head) = parse_f64(head, "PWAT").unwrap_or((none(), head));
        let (totl, head) = parse_f64(head, "TOTL").unwrap_or((none(), head));
        let (cape, head) = parse_f64(head, "CAPE").unwrap_or((none(), head));
        let (lclt, head) = parse_f64(head, "LCLT").unwrap_or((none(), head));
        let (cins, head) = parse_f64(head, "CINS").unwrap_or((none(), head));
        let (eqlv, head) = parse_f64(head, "EQLV").unwrap_or((none(), head));
        let (lfct, head) = parse_f64(head, "LFCT").unwrap_or((none(), head));
        let (brch, _) = parse_f64(head, "BRCH").unwrap_or((none(), head));

        Ok(Indexes {
            show: show.map_t(CelsiusDiff),
            li: lift.map_t(CelsiusDiff),
            swet,
            kinx: kinx.map_t(Celsius),
            lclp: lclp.map_t(HectoPascal),
            pwat: pwat.map_t(Mm),
            totl,
            cape: cape.map_t(JpKg),
            lclt: lclt.map_t(Kelvin),
            cins: cins.map_t(JpKg),
            eqlv: eqlv.map_t(HectoPascal),
            lfc: lfct.map_t(HectoPascal),
            brch,
        })
    }
}

#[test]
fn test_indexes_parse() {
    use optional::some;

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

    assert_eq!(show, some(CelsiusDiff(8.12)));
    assert_eq!(li, some(CelsiusDiff(8.0)));
    assert_eq!(swet, some(39.08));
    assert_eq!(kinx, some(Celsius(14.88)));
    assert_eq!(lclp, some(HectoPascal(780.77)));
    assert_eq!(pwat, some(Mm(9.28)));
    assert_eq!(totl, some(39.55));
    assert_eq!(cape, some(JpKg(0.00)));
    assert_eq!(lclt, some(Kelvin(272.88)));
    assert_eq!(cins, some(JpKg(0.00)));
    assert!(eqlv.is_none());
    assert!(lfc.is_none());
    assert_eq!(brch, some(0.00));

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

    assert_eq!(show, some(CelsiusDiff(9.67)));
    assert_eq!(li, some(CelsiusDiff(9.84)));
    assert_eq!(swet, some(33.41));
    assert_eq!(kinx, some(Celsius(3.88)));
    assert_eq!(lclp, some(HectoPascal(822.95)));
    assert_eq!(pwat, some(Mm(9.52)));
    assert_eq!(totl, some(37.25));
    assert_eq!(cape, some(JpKg(0.00)));
    assert_eq!(lclt, some(Kelvin(273.49)));
    assert_eq!(cins, some(JpKg(0.00)));
    assert!(eqlv.is_none());
    assert!(lfc.is_none());
    assert_eq!(brch, some(0.00));
}
