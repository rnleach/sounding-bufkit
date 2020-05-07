//! Module for reading a bufkit file and breaking it into smaller pieces for parsing later.
use super::surface::SurfaceData;
use super::upper_air::UpperAir;
use crate::parse_util::check_missing_i32;
use metfor::{MetersPSec, Quantity, WindUV};
use optional::Optioned;
use sounding_analysis::{PrecipType, Sounding, StationInfo};
use std::collections::HashMap;

#[allow(clippy::needless_pass_by_value)]
pub fn combine_data(
    ua: UpperAir,
    sd: SurfaceData,
    fname: &str,
) -> (Sounding, HashMap<&'static str, f64>) {
    let coords: Option<(f64, f64)> = ua
        .lat
        .into_option()
        .and_then(|lat| ua.lon.into_option().map(|lon| (lat, lon)));

    let station =
        StationInfo::new_with_values(check_missing_i32(ua.num), ua.id, coords, ua.elevation);

    let snd = Sounding::new()
        .with_source_description(fname.to_owned())
        .with_station_info(station)
        .with_valid_time(ua.valid_time)
        .with_lead_time(check_missing_i32(ua.lead_time))
        // Upper air
        .with_pressure_profile(ua.pressure)
        .with_temperature_profile(ua.temperature)
        .with_wet_bulb_profile(ua.wet_bulb)
        .with_dew_point_profile(ua.dew_point)
        .with_theta_e_profile(ua.theta_e)
        .with_wind_profile(ua.wind)
        .with_pvv_profile(ua.omega)
        .with_height_profile(ua.height)
        .with_cloud_fraction_profile(ua.cloud_fraction)
        // Surface data
        .with_mslp(sd.mslp)
        .with_sfc_temperature(sd.temperature)
        .with_sfc_dew_point(sd.dewpoint)
        .with_station_pressure(sd.station_pres)
        .with_low_cloud(sd.low_cloud)
        .with_mid_cloud(sd.mid_cloud)
        .with_high_cloud(sd.hi_cloud)
        .with_sfc_wind(sd.wind);

    macro_rules! check_and_add {
        ($opt:expr, $key:expr, $hash_map:ident) => {
            if let Some(val) = $opt.into_option() {
                $hash_map.insert($key, val.unpack());
            }
        };
    }

    let mut bufkit_anal: HashMap<&'static str, f64> = HashMap::new();

    // Add some profile indexes.
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
    check_and_add!(sd.p01, "Precipitation1HrMm", bufkit_anal);
    check_and_add!(sd.c01, "ConvectivePrecip1HrMm", bufkit_anal);
    check_and_add!(sd.lyr_2_soil_temp, "Layer2SoilTemp", bufkit_anal);
    check_and_add!(sd.snow_ratio, "SnowRatio", bufkit_anal);
    check_and_add!(sd.visibility, "VisibilityKm", bufkit_anal);
    check_and_add!(sd.srh, "StormRelativeHelicity", bufkit_anal);

    if let Some(WindUV {
        u: MetersPSec(u),
        v: MetersPSec(v),
    }) = sd.storm_motion.into_option()
    {
        bufkit_anal.insert("StormMotionUMps", u);
        bufkit_anal.insert("StormMotionVMps", v);
    }

    // Get the Wx symbol code from bufkit and translate it into the kind that is used in
    // sounding-analysis.
    let wx_code: Optioned<f64> = derived_wx_code(
        sd.wx_sym_cod.map(|code| code as u8),
        sd.rain_type,
        sd.snow_type,
        sd.fzra_type,
        sd.ice_pellets_type,
    )
    .map(|p_type| p_type as u8 as f64)
    .into();
    check_and_add!(wx_code, "WxSymbolCode", bufkit_anal);

    (snd, bufkit_anal)
}
fn derived_wx_code(
    wx_code: Option<u8>,
    is_rain: Option<bool>,
    is_snow: Option<bool>,
    is_fzra: Option<bool>,
    is_ip: Option<bool>,
) -> Option<PrecipType> {
    match wx_code {
        Some(60) => Some(PrecipType::LightRain),
        Some(66) => Some(PrecipType::LightFreezingRain),
        Some(70) => Some(PrecipType::LightSnow),
        Some(79) => Some(PrecipType::LightIcePellets),
        _ => return None,
    }
    .or_else(|| {
        is_rain.and_then(|isra| {
            if isra {
                Some(PrecipType::LightRain)
            } else {
                None
            }
        })
    })
    .or_else(|| {
        is_snow.and_then(|issn| {
            if issn {
                Some(PrecipType::LightSnow)
            } else {
                None
            }
        })
    })
    .or_else(|| {
        is_fzra.and_then(|isfz| {
            if isfz {
                Some(PrecipType::LightFreezingRain)
            } else {
                None
            }
        })
    })
    .or_else(|| {
        is_ip.and_then(|isip| {
            if isip {
                Some(PrecipType::LightIcePellets)
            } else {
                None
            }
        })
    })
}
